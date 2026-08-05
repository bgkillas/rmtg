use bevy::log::info;
use bevy::prelude::{Component, PopulatedMessageReader, Resource};
use bevy_ecs::observer::On;
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_p2p::events::{ConnectFailed, PeerConnected, PeerDisconnected};
use bevy_p2p::iroh::EndpointId;
use bevy_p2p::message::MessageReceived;
use rustc_hash::FxBuildHasher;
use std::collections::HashMap;
#[derive(Encode, Decode)]
pub enum Msg {
    Empty,
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
pub fn receive_message(mut reader: PopulatedMessageReader<MessageReceived<Msg>>) {
    for msg in reader.read() {
        match &msg.message {
            Msg::Empty => {}
        }
    }
}
#[derive(Component, Default, Clone, Copy)]
pub struct Peer {
    pub id: u32,
}
impl Peer {
    #[must_use]
    pub fn new(id: u32) -> Self {
        Peer { id }
    }
}
#[derive(Resource, Default)]
pub struct Peers {
    pub my_id: Option<Peer>,
    pub peer_to_id: HashMap<EndpointId, Peer, FxBuildHasher>,
    pub id_to_peer: HashMap<Peer, EndpointId, FxBuildHasher>,
}
