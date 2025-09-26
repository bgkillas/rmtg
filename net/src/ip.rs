use crate::{Client, ClientCallback, ClientTrait, ClientType, Message, PeerId, Reliability};
use eyre::eyre;
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc::channel;
use tangled::{NetworkEvent, Peer};
use tokio::runtime::Runtime;
pub const DEFAULT_PORT: u16 = 5143;
pub(crate) struct IpClient {
    pub(crate) peer: Peer,
    pub(crate) peer_connected: ClientCallback,
    pub(crate) peer_disconnected: ClientCallback,
    connected: bool,
}
impl IpClient {
    pub(crate) fn host(
        socket_addr: SocketAddr,
        peer_connected: ClientCallback,
        peer_disconnected: ClientCallback,
    ) -> eyre::Result<Self> {
        Ok(Self {
            peer: Peer::host(socket_addr, None)?,
            peer_connected,
            peer_disconnected,
            connected: true,
        })
    }
    pub(crate) fn join(
        socket_addr: SocketAddr,
        peer_connected: ClientCallback,
        peer_disconnected: ClientCallback,
    ) -> eyre::Result<Self> {
        Ok(Self {
            peer: Peer::connect(socket_addr, None)?,
            peer_connected,
            peer_disconnected,
            connected: false,
        })
    }
    pub(crate) fn recv<F>(&mut self, mut f: F)
    where
        F: FnMut(&dyn ClientTrait, Message),
    {
        if self.connected {
            for n in self.peer.recv() {
                match n {
                    NetworkEvent::Message(m) => f(
                        self,
                        Message {
                            src: m.src.into(),
                            data: m.data,
                        },
                    ),
                    NetworkEvent::PeerConnected(peer) => {
                        if let Some(mut c) = self.peer_connected.take() {
                            c(self, peer.into());
                            self.peer_connected = Some(c);
                        }
                    }
                    NetworkEvent::PeerDisconnected(peer) => {
                        if let Some(mut d) = self.peer_disconnected.take() {
                            d(self, peer.into());
                            self.peer_disconnected = Some(d);
                        }
                    }
                }
            }
        }
    }
    pub(crate) fn update(&mut self) {
        if !self.connected && self.peer.my_id().is_some() {
            self.connected = true
        }
    }
}
impl ClientTrait for IpClient {
    fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        if self.connected {
            self.peer.send(dest.into(), data, reliability.into())?;
        }
        Ok(())
    }
    fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        if self.connected {
            self.peer.broadcast(data, reliability.into())?;
        }
        Ok(())
    }
    fn my_id(&self) -> PeerId {
        self.peer.my_id().unwrap().into()
    }
    fn host_id(&self) -> PeerId {
        PeerId(0)
    }
    fn is_host(&self) -> bool {
        self.peer.my_id().unwrap() == tangled::PeerId(0)
    }
    fn peer_len(&self) -> usize {
        self.peer.iter_peer_ids().count()
    }
}
impl From<Reliability> for tangled::Reliability {
    fn from(value: Reliability) -> Self {
        match value {
            Reliability::Reliable => tangled::Reliability::Reliable,
            Reliability::Unreliable => tangled::Reliability::Unreliable,
        }
    }
}
impl From<tangled::PeerId> for PeerId {
    fn from(value: tangled::PeerId) -> Self {
        Self(value.0.into())
    }
}
impl From<PeerId> for tangled::PeerId {
    fn from(value: PeerId) -> Self {
        Self(value.raw() as u16)
    }
}
impl Client {
    pub fn host_ip(
        &mut self,
        peer_connected: ClientCallback,
        peer_disconnected: ClientCallback,
    ) -> eyre::Result<()> {
        let socket = SocketAddr::new("::".parse()?, DEFAULT_PORT);
        self.client = ClientType::Ip(IpClient::host(socket, peer_connected, peer_disconnected)?);
        Ok(())
    }
    pub fn join_ip(
        &mut self,
        addr: IpAddr,
        peer_connected: ClientCallback,
        peer_disconnected: ClientCallback,
    ) -> eyre::Result<()> {
        let socket = SocketAddr::new(addr, DEFAULT_PORT);
        self.client = ClientType::Ip(IpClient::join(socket, peer_connected, peer_disconnected)?);
        Ok(())
    }
    pub fn host_ip_runtime(
        &mut self,
        peer_connected: ClientCallback,
        peer_disconnected: ClientCallback,
        runtime: &Runtime,
    ) -> eyre::Result<()> {
        let socket = SocketAddr::new("::".parse()?, DEFAULT_PORT);
        let (tx, rx) = channel();
        runtime.spawn(
            async move { tx.send(IpClient::host(socket, peer_connected, peer_disconnected)) },
        );
        if let Ok(client) = rx.recv() {
            self.client = ClientType::Ip(client?);
            Ok(())
        } else {
            Err(eyre!("not found"))
        }
    }
    pub fn join_ip_runtime(
        &mut self,
        addr: IpAddr,
        peer_connected: ClientCallback,
        peer_disconnected: ClientCallback,
        runtime: &Runtime,
    ) -> eyre::Result<()> {
        let socket = SocketAddr::new(addr, DEFAULT_PORT);
        let (tx, rx) = channel();
        runtime.spawn(
            async move { tx.send(IpClient::join(socket, peer_connected, peer_disconnected)) },
        );
        if let Ok(client) = rx.recv() {
            self.client = ClientType::Ip(client?);
            Ok(())
        } else {
            Err(eyre!("not found"))
        }
    }
}
