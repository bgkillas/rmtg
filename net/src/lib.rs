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
    pub fn recv<F>(&mut self, mut f: F)
    where
        F: FnMut(&dyn ClientTrait, Message),
    {
        match &mut self.client {
            ClientType::None => {}
            #[cfg(feature = "steam")]
            ClientType::Steam(client) => client.recv().for_each(|m| f(client, m)),
            #[cfg(feature = "tangled")]
            ClientType::Ip(client) => client.clone().recv().for_each(|m| f(client, m)),
        }
    }
}
pub trait ClientTrait {
    fn send_message(&self, dest: PeerId, data: &[u8], reliability: Reliability)
    -> eyre::Result<()>;
    fn broadcast(&self, data: &[u8], reliability: Reliability) -> eyre::Result<()>;
    fn my_id(&self) -> PeerId;
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
        });
        #[cfg(feature = "steam")]
        app.add_systems(bevy_app::First, update);
    }
}
#[cfg(all(feature = "bevy", feature = "steam"))]
pub fn update(mut client: bevy_ecs::system::ResMut<Client>) {
    client.update()
}
