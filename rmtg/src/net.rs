use crate::camera_indicator::move_camera;
use crate::spatial::Spatial;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{Component, PopulatedMessageReader, Resource};
use bevy_ecs::observer::On;
use bevy_ecs::system::Commands;
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_p2p::events::{ConnectFailed, PeerConnected, PeerDisconnected};
use bevy_p2p::iroh::EndpointId;
use bevy_p2p::iroh_res::Compression;
use bevy_p2p::message::{MessageReceived, Net};
use enum_map::Enum;
use importer::coder::DataCoder;
use rand::RngExt as _;
use rand::rngs::StdRng;
use rustc_hash::FxBuildHasher;
use std::collections::HashMap;
#[derive(Encode, Decode)]
pub enum Msg {
    Camera {
        #[bitcode(with = "DataCoder<Vec3>")]
        camera: Vec3,
        #[bitcode(with = "DataCoder<Vec3>")]
        cursor: Vec3,
    },
}
pub fn net_update(net: Net<Msg>, spatial: Spatial) {
    if let Some((_, cursor, _)) = spatial.ray() {
        let camera = spatial.camera.transform.translation;
        net.broadcast(Compression::None, Msg::Camera { camera, cursor });
    }
}
pub fn receive_message(
    mut reader: PopulatedMessageReader<MessageReceived<Msg>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        match &msg.message {
            &Msg::Camera { camera, cursor } => {
                commands.run_system_cached_with(move_camera, (msg.peer, camera, cursor));
            }
        }
    }
}
#[derive(Component, Clone, Copy, Encode, Decode)]
pub struct GlobalId {
    pub id: u64,
}
impl Default for GlobalId {
    fn default() -> Self {
        let mut rng = rand::make_rng::<StdRng>();
        let id = rng.random();
        Self { id }
    }
}
#[derive(Component, Clone, Copy)]
pub struct Endpoint {
    pub peer: EndpointId,
}
impl From<EndpointId> for Endpoint {
    fn from(peer: EndpointId) -> Self {
        Self { peer }
    }
}
#[derive(Enum, Component, Default, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum Peer {
    #[default]
    Zero,
    One,
    Two,
    Three,
}
impl Peer {
    pub fn new(id: usize) -> Self {
        match id {
            0 => Peer::Zero,
            1 => Peer::One,
            2 => Peer::Two,
            3 => Peer::Three,
            _ => unreachable!(),
        }
    }
}
#[derive(Resource, Default)]
pub struct Peers {
    pub my_endpoint: Option<Endpoint>,
    pub my_id: Option<Peer>,
    pub peer_to_id: HashMap<EndpointId, Peer, FxBuildHasher>,
    pub id_to_peer: HashMap<Peer, EndpointId, FxBuildHasher>,
}
pub fn connect_failed(event: On<ConnectFailed>) {
    info!("{} failed", event.peer.fmt_short());
}
pub fn on_connect(event: On<PeerConnected>) {
    info!("{} connect", event.peer.fmt_short());
}
pub fn on_disconnect(event: On<PeerDisconnected>) {
    info!("{} disconnect", event.peer.fmt_short());
}
