use crate::spatial::Spatial;
use bevy::log::info;
use bevy::math::Vec3;
use bevy::prelude::{Component, PopulatedMessageReader, Resource};
use bevy_ecs::observer::On;
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_p2p::events::{ConnectFailed, PeerConnected, PeerDisconnected};
use bevy_p2p::iroh::EndpointId;
use bevy_p2p::message::{MessageReceived, Net};
use importer::coder::DataCoder;
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
        let camera = spatial.camera.1.translation;
        net.broadcast(Msg::Camera { camera, cursor });
    }
}
pub fn receive_message(mut reader: PopulatedMessageReader<MessageReceived<Msg>>) {
    for msg in reader.read() {
        match &msg.message {
            Msg::Camera { camera, cursor } => {
                _ = camera;
                _ = cursor;
            }
        }
    }
}
#[derive(Component, Clone, Copy)]
pub struct Endpoint {
    pub peer: EndpointId,
}
#[derive(Component, Default, Clone, Copy)]
pub struct Peer {
    pub id: usize,
}
impl Peer {
    #[must_use]
    pub fn new(id: usize) -> Self {
        Peer { id }
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
