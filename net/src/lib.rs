#[cfg(feature = "tangled")]
mod ip;
#[cfg(feature = "steam")]
mod steam;
#[cfg(feature = "tangled")]
use crate::ip::IpClient;
#[cfg(feature = "steam")]
use crate::steam::SteamClient;
#[cfg(feature = "bevy")]
use bevy_app::{App, Plugin};
#[cfg(feature = "bevy")]
use bevy_ecs::resource::Resource;
type ClientCallback = Option<Box<dyn FnMut(&dyn ClientTrait, PeerId) + Send + Sync + 'static>>;
pub struct Message {
    pub src: PeerId,
    pub data: Box<[u8]>,
}
#[derive(Copy, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum Reliability {
    Reliable,
    Unreliable,
}
#[derive(Copy, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct PeerId(pub u64);
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
            ClientType::Ip(client) => client.my_id().raw() as u16,
            ClientType::None => 0,
        }
    }
    pub fn my_id(&self) -> PeerId {
        match &self.client {
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.my_id,
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.my_id(),
            ClientType::None => PeerId(0),
        }
    }
    pub fn send_message(
        &self,
        dest: PeerId,
        data: &[u8],
        reliability: Reliability,
    ) -> eyre::Result<()> {
        match &self.client {
            ClientType::None => {}
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.send_message(dest, data, reliability)?,
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.send_message(dest, data, reliability)?,
        }
        Ok(())
    }
    pub fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()> {
        match &self.client {
            ClientType::None => {}
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.broadcast(data, reliability)?,
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.broadcast(data, reliability)?,
        }
        Ok(())
    }
    pub fn recv<F>(&mut self, f: F)
    where
        F: FnMut(&dyn ClientTrait, Message),
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
}
pub trait ClientTrait {
    fn send_message(&self, dest: PeerId, data: &[u8], reliability: Reliability)
    -> eyre::Result<()>;
    fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()>;
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
        .broadcast(&[0, 1, 5, 3], Reliability::Reliable)
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let mut has = false;
    peer1.recv(|_, m| has = *m.data == [0, 1, 5, 3]);
    assert!(has);
    let mut has = false;
    host.recv(|_, m| has = *m.data == [0, 1, 5, 3]);
    assert!(has)
}
