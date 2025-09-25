use crate::{Client, ClientTrait, ClientType, Message, PeerId, Reliability};
use std::net::SocketAddr;
use tangled::{NetworkEvent, Peer};
pub const DEFAULT_PORT: u16 = 5143;
#[derive(Clone)]
pub(crate) struct IpClient {
    pub(crate) peer: Peer,
}
impl IpClient {
    pub(crate) fn host(socket_addr: SocketAddr) -> eyre::Result<Self> {
        Ok(Self {
            peer: Peer::host(socket_addr, None)?,
        })
    }
    pub(crate) fn join(socket_addr: SocketAddr) -> eyre::Result<Self> {
        Ok(Self {
            peer: Peer::connect(socket_addr, None)?,
        })
    }
    pub(crate) fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        self.peer.send(dest.into(), data, reliability.into())?;
        Ok(())
    }
    pub(crate) fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        self.peer.broadcast(data, reliability.into())?;
        Ok(())
    }
    pub(crate) fn recv(&mut self) -> impl Iterator<Item = Message> + use<'_> {
        self.peer.recv().filter_map(|n| {
            if let NetworkEvent::Message(m) = n {
                Some(Message {
                    src: m.src.into(),
                    data: m.data,
                })
            } else {
                None
            }
        })
    }
}
impl ClientTrait for IpClient {
    fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        self.peer.send(dest.into(), data, reliability.into())?;
        Ok(())
    }
    fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        self.peer.broadcast(data, reliability.into())?;
        Ok(())
    }
    fn my_id(&self) -> PeerId {
        self.peer.my_id().unwrap().into()
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
    pub fn host_ip(&mut self) -> eyre::Result<()> {
        let socket = SocketAddr::new("::".parse()?, DEFAULT_PORT);
        self.client = ClientType::Ip(IpClient::host(socket)?);
        Ok(())
    }
    pub fn join_ip(&mut self, addr: &str) -> eyre::Result<()> {
        let socket = SocketAddr::new(addr.parse()?, DEFAULT_PORT);
        self.client = ClientType::Ip(IpClient::join(socket)?);
        Ok(())
    }
}
