mod ip;
mod steam;
use crate::ip::IpClient;
use crate::steam::SteamClient;
use bevy_app::{App, First, Plugin};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::ResMut;
use std::net::SocketAddr;
use steamworks::networking_messages::SessionRequest;
use steamworks::networking_types::SendFlags;
use steamworks::{LobbyId, SteamId};
#[cfg(feature = "tangled")]
pub const DEFAULT_PORT: u16 = 5143;
pub struct Message {
    pub src: PeerId,
    pub data: Box<[u8]>,
}
#[derive(Copy, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum Reliability {
    Reliable,
    Unreliable,
}
#[cfg(feature = "tangled")]
impl From<Reliability> for tangled::Reliability {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::Reliable => tangled::Reliability::Reliable,
            Reliability::Unreliable => tangled::Reliability::Unreliable,
        }
    }
}
#[cfg(feature = "steam")]
impl From<Reliability> for SendFlags {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::Reliable => SendFlags::RELIABLE,
            Reliability::Unreliable => SendFlags::UNRELIABLE,
        }
    }
}
#[derive(Copy, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct PeerId(pub u64);
impl PeerId {
    pub fn raw(&self) -> u64 {
        self.0
    }
}
#[cfg(feature = "steam")]
impl From<SteamId> for PeerId {
    fn from(value: SteamId) -> Self {
        Self(value.raw())
    }
}
#[cfg(feature = "tangled")]
impl From<tangled::PeerId> for PeerId {
    fn from(value: tangled::PeerId) -> Self {
        Self(value.0.into())
    }
}
#[cfg(feature = "steam")]
impl From<PeerId> for SteamId {
    fn from(value: PeerId) -> Self {
        Self::from_raw(value.0)
    }
}
#[cfg(feature = "tangled")]
impl From<PeerId> for tangled::PeerId {
    fn from(value: PeerId) -> Self {
        Self(value.0 as u16)
    }
}
pub(crate) enum ClientType {
    None,
    #[cfg(feature = "steam")]
    Steam(SteamClient),
    #[cfg(feature = "tangled")]
    Ip(IpClient),
}
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct Client {
    client: ClientType,
    #[cfg(feature = "steam")]
    app_id: u32,
}
impl Client {
    pub fn new(#[cfg(feature = "steam")] app_id: u32) -> Self {
        Self {
            #[cfg(feature = "steam")]
            app_id,
            client: ClientType::None,
        }
    }
    pub fn my_num(&self) -> u16 {
        match &self.client {
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.my_num,
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.peer.my_id().unwrap_or(tangled::PeerId(0)).0,
            ClientType::None => 0,
        }
    }
    pub fn my_id(&self) -> PeerId {
        match &self.client {
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.my_id,
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.peer.my_id().unwrap().into(),
            ClientType::None => PeerId(0),
        }
    }
    #[cfg(feature = "steam")]
    pub fn host_steam(&mut self) -> eyre::Result<()> {
        if !matches!(self.client, ClientType::Steam(_)) {
            self.client = ClientType::Steam(SteamClient::new(self.app_id)?);
        }
        if let ClientType::Steam(client) = &mut self.client {
            client.host()?;
        }
        Ok(())
    }
    #[cfg(feature = "tangled")]
    pub fn host_ip(&mut self) -> eyre::Result<()> {
        let socket = SocketAddr::new("::".parse()?, DEFAULT_PORT);
        self.client = ClientType::Ip(IpClient::host(socket)?);
        Ok(())
    }
    #[cfg(feature = "steam")]
    pub fn join_steam(&mut self, lobby: LobbyId) -> eyre::Result<()> {
        if !matches!(self.client, ClientType::Steam(_)) {
            self.client = ClientType::Steam(SteamClient::new(self.app_id)?);
        }
        if let ClientType::Steam(client) = &mut self.client {
            client.join(lobby);
        }
        Ok(())
    }
    #[cfg(feature = "tangled")]
    pub fn join_ip(&mut self, addr: &str) -> eyre::Result<()> {
        let socket = SocketAddr::new(addr.parse()?, DEFAULT_PORT);
        self.client = ClientType::Ip(IpClient::join(socket)?);
        Ok(())
    }
    pub fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        match &self.client {
            ClientType::None => {}
            ClientType::Steam(client) => client.send_message(dest, data, reliability)?,
            ClientType::Ip(client) => client.send_message(dest, data, reliability)?,
        }
        Ok(())
    }
    pub fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        match &self.client {
            ClientType::None => {}
            ClientType::Steam(client) => client.broadcast(data, reliability)?,
            ClientType::Ip(client) => client.broadcast(data, reliability)?,
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
    pub fn update(&mut self) {
        if let ClientType::Steam(client) = &mut self.client {
            client.update()
        }
    }
    pub fn recv(&mut self) -> Box<dyn Iterator<Item = Message> + '_> {
        match &mut self.client {
            ClientType::None => Box::new(std::iter::empty()),
            ClientType::Steam(client) => Box::new(client.recv()),
            ClientType::Ip(client) => Box::new(client.recv()),
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
}
#[cfg(feature = "bevy")]
impl Plugin for Client {
    fn build(&self, app: &mut App) {
        app.insert_resource(Self {
            #[cfg(feature = "steam")]
            app_id: self.app_id,
            #[cfg(feature = "steam")]
            client: SteamClient::new(self.app_id)
                .map(ClientType::Steam)
                .unwrap_or(ClientType::None),
            #[cfg(not(feature = "steam"))]
            client: ClientType::None,
        })
        .add_systems(First, update);
    }
}
pub fn update(mut client: ResMut<Client>) {
    client.update()
}
