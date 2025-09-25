use crate::{Client, ClientTrait, ClientType, Message, PeerId, Reliability};
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use steamworks::networking_messages::SessionRequest;
use steamworks::networking_sockets::{ListenSocket, NetConnection, NetPollGroup};
use steamworks::networking_types::{
    ListenSocketEvent, NetConnectionStatusChanged, NetworkingConnectionState, NetworkingIdentity,
    SendFlags,
};
use steamworks::{CallbackResult, GameLobbyJoinRequested, LobbyId, LobbyType, SteamId};
pub(crate) struct Connection {
    pub(crate) net: NetConnection,
    pub(crate) connected: bool,
}
pub(crate) struct SteamClient {
    pub(crate) steam_client: steamworks::Client,
    pub(crate) my_id: PeerId,
    pub(crate) host_id: PeerId,
    pub(crate) lobby_id: LobbyId,
    pub(crate) connections: HashMap<PeerId, Connection>,
    pub(crate) poll_group: NetPollGroup,
    pub(crate) is_host: bool,
    pub(crate) listen_socket: MaybeUninit<Mutex<ListenSocket>>,
    pub(crate) my_num: u16,
    rx: Arc<Mutex<Receiver<LobbyId>>>,
    tx: Arc<Mutex<Sender<LobbyId>>>,
}
impl SteamClient {
    fn reset(&mut self) {
        self.host_id = PeerId(0);
        self.lobby_id = LobbyId::from_raw(0);
        self.connections = Default::default();
        self.is_host = false;
        self.my_num = 0;
        self.listen_socket = MaybeUninit::uninit();
    }
    pub(crate) fn new(app_id: u32) -> eyre::Result<Self> {
        let steam_client = steamworks::Client::init_app(app_id)?;
        steam_client.networking_utils().init_relay_network_access();
        let poll_group = steam_client.networking_sockets().create_poll_group();
        let my_id = steam_client.user().steam_id().into();
        let (tx, rx) = channel();
        Ok(Self {
            steam_client,
            my_id,
            host_id: PeerId(0),
            lobby_id: LobbyId::from_raw(0),
            connections: Default::default(),
            poll_group,
            my_num: 0,
            is_host: false,
            listen_socket: MaybeUninit::uninit(),
            rx: Arc::new(rx.into()),
            tx: Arc::new(tx.into()),
        })
    }
    pub(crate) fn host(&mut self) -> eyre::Result<()> {
        self.reset();
        self.host_id = self.my_id;
        self.is_host = true;
        self.listen_socket = MaybeUninit::new(
            self.steam_client
                .networking_sockets()
                .create_listen_socket_p2p(0, None)?
                .into(),
        );
        let tx = self.tx.clone();
        self.steam_client
            .matchmaking()
            .create_lobby(LobbyType::FriendsOnly, 250, move |s| {
                if let Ok(s) = s {
                    let _ = tx.lock().unwrap().send(s);
                }
            });
        Ok(())
    }
    pub(crate) fn join(&mut self, id: LobbyId) {
        self.reset();
        let tx = self.tx.clone();
        self.steam_client.matchmaking().join_lobby(id, move |s| {
            if let Ok(s) = s {
                let _ = tx.lock().unwrap().send(s);
            }
        })
    }
    pub(crate) fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        if let Some(con) = self.connections.get(&dest)
            && con.connected
        {
            con.net.send_message(data, reliability.into())?;
        }
        Ok(())
    }
    pub(crate) fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        for con in self.connections.values() {
            if con.connected {
                con.net.send_message(data, reliability.into())?;
            }
        }
        Ok(())
    }
    pub(crate) fn recv(&mut self) -> impl Iterator<Item = Message> + use<> {
        self.poll_group
            .receive_messages(1024)
            .into_iter()
            .map(|m| Message {
                src: m.identity_peer().steam_id().unwrap().into(),
                data: m.data().into(),
            })
    }
    fn connect(&mut self, id: SteamId) {
        let peer_identity = NetworkingIdentity::new_steam_id(id);
        let connection = self
            .steam_client
            .networking_sockets()
            .connect_p2p(peer_identity, 0, None)
            .unwrap();
        connection.set_poll_group(&self.poll_group);
        self.connections.insert(
            id.into(),
            Connection {
                net: connection,
                connected: false,
            },
        );
    }
    pub(crate) fn update(&mut self) {
        let recv = self.rx.clone();
        let recv = recv.lock().unwrap();
        while let Ok(event) = recv.try_recv() {
            self.lobby_id = event;
            if !self.is_host {
                let matchmaking = self.steam_client.matchmaking();
                let owner = matchmaking.lobby_owner(event);
                self.host_id = owner.into();
                for id in matchmaking.lobby_members(event) {
                    if id != self.my_id.into() {
                        self.connect(id)
                    }
                }
            }
        }
        self.steam_client
            .clone()
            .process_callbacks(|callback| match callback {
                CallbackResult::GameLobbyJoinRequested(GameLobbyJoinRequested {
                    lobby_steam_id,
                    ..
                }) => self.join(lobby_steam_id),
                CallbackResult::NetConnectionStatusChanged(NetConnectionStatusChanged {
                    connection_info,
                    ..
                }) => {
                    if matches!(
                        connection_info.state(),
                        Ok(NetworkingConnectionState::Connected)
                    ) {
                        let peer = connection_info
                            .identity_remote()
                            .unwrap()
                            .steam_id()
                            .unwrap();
                        if let Some(con) = self.connections.get_mut(&peer.into()) {
                            println!("connected to {peer:?}");
                            con.connected = true
                        }
                    }
                }
                _ => {}
            });
        if self.is_host {
            let listen = unsafe { self.listen_socket.assume_init_ref() }
                .lock()
                .unwrap();
            while let Some(event) = listen.try_receive_event() {
                match event {
                    ListenSocketEvent::Connecting(event) => {
                        println!("connecting to someone");
                        event.accept().unwrap();
                    }
                    ListenSocketEvent::Connected(event) => {
                        let id = event.remote().steam_id().unwrap();
                        println!("connected to {id:?}");
                        let connection = event.take_connection();
                        connection.set_poll_group(&self.poll_group);
                        /*connection
                        .send_message(&encode(&Packet::SetUser(peers.count)), SendFlags::RELIABLE)
                        .unwrap();todo */
                        let connection = Connection {
                            net: connection,
                            connected: true,
                        };
                        self.connections.insert(id.into(), connection);
                    }
                    ListenSocketEvent::Disconnected(event) => {
                        let id = event.remote().steam_id().unwrap();
                        self.connections.remove(&id.into());
                        println!("disconnected from {id:?}");
                    }
                }
            }
        }
    }
}
impl ClientTrait for SteamClient {
    fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        if let Some(con) = self.connections.get(&dest)
            && con.connected
        {
            con.net.send_message(data, reliability.into())?;
        }
        Ok(())
    }
    fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        for con in self.connections.values() {
            if con.connected {
                con.net.send_message(data, reliability.into())?;
            }
        }
        Ok(())
    }
    fn my_id(&self) -> PeerId {
        self.my_id
    }
}
impl From<Reliability> for SendFlags {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::Reliable => SendFlags::RELIABLE,
            Reliability::Unreliable => SendFlags::UNRELIABLE,
        }
    }
}
impl From<SteamId> for PeerId {
    fn from(value: SteamId) -> Self {
        Self(value.raw())
    }
}
impl From<PeerId> for SteamId {
    fn from(value: PeerId) -> Self {
        Self::from_raw(value.raw())
    }
}
impl Client {
    pub fn host_steam(&mut self) -> eyre::Result<()> {
        self.init_steam()?;
        if let ClientType::Steam(client) = &mut self.client {
            client.host()?;
        }
        Ok(())
    }
    pub fn join_steam(&mut self, lobby: LobbyId) -> eyre::Result<()> {
        self.init_steam()?;
        if let ClientType::Steam(client) = &mut self.client {
            client.join(lobby);
        }
        Ok(())
    }
    pub fn args(&self) -> String {
        if let ClientType::Steam(client) = &self.client {
            client.steam_client.apps().launch_command_line()
        } else {
            String::new()
        }
    }
    pub fn session_request_callback(&self, f: impl FnMut(SessionRequest) + Send + 'static) {
        if let ClientType::Steam(client) = &self.client {
            client
                .steam_client
                .networking_messages()
                .session_request_callback(f);
        }
    }
    pub fn flush(&mut self) {
        if let ClientType::Steam(client) = &mut self.client {
            client.connections.values_mut().for_each(|c| {
                if c.connected {
                    c.net.flush_messages().unwrap();
                }
            })
        }
    }
    pub fn update(&mut self) {
        if let ClientType::Steam(client) = &mut self.client {
            client.update()
        }
    }
    pub fn init_steam(&mut self) -> eyre::Result<()> {
        if !matches!(self.client, ClientType::Steam(_)) {
            self.client = ClientType::Steam(SteamClient::new(self.app_id)?);
        }
        Ok(())
    }
}
