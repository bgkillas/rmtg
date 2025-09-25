use crate::{ClientTrait, Message, PeerId, Reliability};
use std::net::SocketAddr;
use tangled::{NetworkEvent, Peer};
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
