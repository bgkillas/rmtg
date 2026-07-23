use bevy::prelude::{Component, PopulatedMessageReader, Resource};
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_p2p::iroh::EndpointId;
use bevy_p2p::message::{ConnectFailed, MessageReceived, PeerConnected, PeerDisconnected};
use rustc_hash::FxHashMap;
#[derive(Encode, Decode)]
pub enum Msg {
    Empty,
}
pub fn connect_failed(mut reader: PopulatedMessageReader<ConnectFailed>) {
    for peer in reader.read() {
        println!("{} failed", peer.peer.fmt_short());
    }
}
pub fn on_connect(mut reader: PopulatedMessageReader<PeerConnected>) {
    for peer in reader.read() {
        println!("{} connect", peer.peer.fmt_short());
    }
}
pub fn on_disconnect(mut reader: PopulatedMessageReader<PeerDisconnected>) {
    for peer in reader.read() {
        println!("{} disconnect", peer.peer.fmt_short());
    }
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
    pub peer_to_id: FxHashMap<EndpointId, Peer>,
    pub id_to_peer: FxHashMap<Peer, EndpointId>,
}
