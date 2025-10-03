#[cfg(feature = "tangled")]
mod ip;
#[cfg(feature = "steam")]
mod steam;
use std::fmt::{Display, Formatter};
#[cfg(feature = "tangled")]
use crate::ip::IpClient;
#[cfg(feature = "steam")]
use crate::steam::SteamClient;
#[cfg(feature = "bevy")]
use bevy_app::{App, Plugin};
#[cfg(feature = "bevy")]
use bevy_ecs::resource::Resource;
use bitcode::{DecodeOwned, Encode};
#[cfg(feature = "steam")]
use steamworks::networking_types::NetConnectionRealTimeInfo;
#[allow(dead_code)]
type ClientCallback = Option<Box<dyn FnMut(ClientTypeRef, PeerId) + Send + Sync + 'static>>;
pub struct Message<T> {
    pub src: PeerId,
    pub data: T,
}
#[derive(Copy, Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum Reliability {
    Reliable,
    Unreliable,
}
#[derive(Copy, Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct PeerId(pub u64);
impl Display for PeerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl PeerId {
    pub fn raw(&self) -> u64 {
        self.0
    }
}
pub(crate) enum ClientType {
    None,
    #[cfg(feature = "steam")]
    Steam(SteamClient),
    #[cfg(feature = "tangled")]
    Ip(IpClient),
}
pub enum ClientTypeRef<'a> {
    #[cfg(feature = "steam")]
    Steam(&'a SteamClient),
    #[cfg(feature = "tangled")]
    Ip(&'a IpClient),
    #[cfg(not(any(feature = "steam", feature = "tangled")))]
    None(&'a u8),
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
    #[allow(unused_variables)]
    pub fn recv<T, F>(&mut self, f: F)
    where
        F: FnMut(ClientTypeRef, Message<T>),
        T: DecodeOwned,
    {
        match &mut self.client {
            ClientType::None => {}
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.recv(f),
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.recv(f),
        }
    }
    pub fn update(&mut self) {
        match &mut self.client {
            ClientType::None => {}
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.update(),
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.update(),
        }
    }
    pub fn info(&self) -> Option<NetworkingInfo> {
        match &self.client {
            ClientType::None => None,
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => Some(client.info()),
            #[cfg(feature = "tangled")]
            ClientType::Ip(_) => None,
        }
    }
}
pub struct NetworkingInfo(#[cfg(feature = "steam")] pub Vec<(PeerId, NetConnectionRealTimeInfo)>);
impl ClientTrait for Client {
    #[allow(unused_variables)]
    fn send_message<T: Encode>(
        &self,
        dest: PeerId,
        data: &T,
        reliability: Reliability,
    ) -> eyre::Result<()> {
        self.client.send_message(dest, data, reliability)
    }
    #[allow(unused_variables)]
    fn broadcast<T: Encode>(&self, data: &T, reliability: Reliability) -> eyre::Result<()> {
        self.client.broadcast(data, reliability)
    }
    fn my_id(&self) -> PeerId {
        self.client.my_id()
    }
    fn host_id(&self) -> PeerId {
        self.client.host_id()
    }
    fn is_host(&self) -> bool {
        self.client.is_host()
    }
    fn peer_len(&self) -> usize {
        self.client.peer_len()
    }
}
impl ClientTrait for ClientType {
    #[allow(unused_variables)]
    fn send_message<T: Encode>(
        &self,
        dest: PeerId,
        data: &T,
        reliability: Reliability,
    ) -> eyre::Result<()> {
        match &self {
            Self::None => {}
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.send_message(dest, data, reliability)?,
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.send_message(dest, data, reliability)?,
        }
        Ok(())
    }
    #[allow(unused_variables)]
    fn broadcast<T: Encode>(&self, data: &T, reliability: Reliability) -> eyre::Result<()> {
        match &self {
            Self::None => {}
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.broadcast(data, reliability)?,
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.broadcast(data, reliability)?,
        }
        Ok(())
    }
    fn my_id(&self) -> PeerId {
        match &self {
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.my_id,
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.my_id(),
            Self::None => PeerId(0),
        }
    }
    fn host_id(&self) -> PeerId {
        match &self {
            Self::None => PeerId(0),
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.host_id(),
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.host_id(),
        }
    }
    fn is_host(&self) -> bool {
        match &self {
            Self::None => true,
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.is_host(),
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.is_host(),
        }
    }
    fn peer_len(&self) -> usize {
        match &self {
            Self::None => 0,
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.peer_len(),
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.peer_len(),
        }
    }
}
impl ClientTrait for ClientTypeRef<'_> {
    #[allow(unused_variables)]
    fn send_message<T: Encode>(
        &self,
        dest: PeerId,
        data: &T,
        reliability: Reliability,
    ) -> eyre::Result<()> {
        match &self {
            #[cfg(not(any(feature = "steam", feature = "tangled")))]
            Self::None(_) => {}
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.send_message(dest, data, reliability)?,
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.send_message(dest, data, reliability)?,
        }
        Ok(())
    }
    #[allow(unused_variables)]
    fn broadcast<T: Encode>(&self, data: &T, reliability: Reliability) -> eyre::Result<()> {
        match &self {
            #[cfg(not(any(feature = "steam", feature = "tangled")))]
            Self::None(_) => {}
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.broadcast(data, reliability)?,
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.broadcast(data, reliability)?,
        }
        Ok(())
    }
    fn my_id(&self) -> PeerId {
        match &self {
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.my_id,
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.my_id(),
            #[cfg(not(any(feature = "steam", feature = "tangled")))]
            Self::None(_) => PeerId(0),
        }
    }
    fn host_id(&self) -> PeerId {
        match &self {
            #[cfg(not(any(feature = "steam", feature = "tangled")))]
            Self::None(_) => PeerId(0),
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.host_id(),
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.host_id(),
        }
    }
    fn is_host(&self) -> bool {
        match &self {
            #[cfg(not(any(feature = "steam", feature = "tangled")))]
            Self::None(_) => true,
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.is_host(),
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.is_host(),
        }
    }
    fn peer_len(&self) -> usize {
        match &self {
            #[cfg(not(any(feature = "steam", feature = "tangled")))]
            Self::None(_) => 0,
            #[cfg(feature = "steam")]
            Self::Steam(client) => client.peer_len(),
            #[cfg(feature = "tangled")]
            Self::Ip(client) => client.peer_len(),
        }
    }
}
pub trait ClientTrait {
    fn send_message<T: Encode>(
        &self,
        dest: PeerId,
        data: &T,
        reliability: Reliability,
    ) -> eyre::Result<()>;
    fn broadcast<T: Encode>(&self, data: &T, reliability: Reliability) -> eyre::Result<()>;
    fn my_id(&self) -> PeerId;
    fn host_id(&self) -> PeerId;
    fn is_host(&self) -> bool;
    fn peer_len(&self) -> usize;
}
#[cfg(feature = "bevy")]
impl Plugin for Client {
    fn build(&self, app: &mut App) {
        app.insert_resource(Self {
            #[cfg(feature = "steam")]
            app_id: self.app_id,
            client: ClientType::None,
        });
        #[cfg(feature = "steam")]
        app.add_systems(bevy_app::First, update);
    }
}
#[cfg(feature = "bevy")]
pub fn update(mut client: bevy_ecs::system::ResMut<Client>) {
    client.update()
}
#[cfg(feature = "tangled")]
#[cfg(test)]
#[tokio::test]
async fn test_ip() {
    let mut host = Client::new(0);
    host.host_ip(None, None).unwrap();
    let mut peer1 = Client::new(0);
    peer1
        .join_ip("127.0.0.1".parse().unwrap(), None, None)
        .unwrap();
    let mut peer2 = Client::new(0);
    peer2
        .join_ip("127.0.0.1".parse().unwrap(), None, None)
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    peer1.update();
    peer2.update();
    peer2
        .broadcast(&[0u8, 1, 5, 3], Reliability::Reliable)
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let mut has = false;
    peer1.recv::<[u8; 4], _>(|_, m| has = m.data == [0, 1, 5, 3]);
    assert!(has);
    let mut has = false;
    host.recv::<[u8; 4], _>(|_, m| has = m.data == [0, 1, 5, 3]);
    assert!(has)
}
