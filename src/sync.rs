use crate::*;
use bevy_steamworks::{Client, SendType, SteamId};
use bitcode::{Decode, Encode, decode, encode};
use lz4_flex::{compress, decompress};
pub fn get_sync(
    client: Res<Client>,
    query: Query<(&SyncObjectMe, &Transform)>,
    count: Res<SyncCount>,
    peers: Res<Peers>,
) {
    let mut v = Vec::with_capacity(count.0);
    for (id, transform) in query {
        v.push((*id, Trans::from(transform)))
    }
    let packet = Packet::Pos(v);
    let bytes = encode(&packet);
    let compressed = compress(&bytes);
    for peer in &peers.0 {
        client
            .networking()
            .send_p2p_packet(*peer, SendType::Reliable, &compressed);
    }
}
pub fn apply_sync(client: Res<Client>, mut query: Query<(&SyncObject, &mut Transform)>) {
    while let Some(size) = client.networking().is_p2p_packet_available() {
        let mut data = vec![0; size];
        if let Some((sender, _)) = client.networking().read_p2p_packet(&mut data) {
            let bytes = decompress(&data, 0).unwrap();
            let packet = decode(&bytes).unwrap();
            match packet {
                Packet::Pos(data) => {
                    let user = sender.raw();
                    for (id, trans) in data {
                        let id = SyncObject { user, id: id.0 };
                        if let Some(mut t) = query
                            .iter_mut()
                            .find_map(|(a, b)| if *a == id { Some(b) } else { None })
                        {
                            *t = trans.into()
                        } else {
                            //todo request
                        }
                    }
                }
            }
        }
    }
}
#[derive(Encode, Decode)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans)>),
}
#[derive(Encode, Decode)]
pub struct Trans {
    pub translation: (u32, u32, u32),
    pub rotation: u128,
}
impl Trans {
    fn from(value: &Transform) -> Self {
        Self {
            translation: unsafe { std::mem::transmute::<Vec3, (u32, u32, u32)>(value.translation) },
            rotation: unsafe { std::mem::transmute::<Quat, u128>(value.rotation) },
        }
    }
}
impl From<Trans> for Transform {
    fn from(value: Trans) -> Self {
        Self {
            translation: unsafe { std::mem::transmute::<(u32, u32, u32), Vec3>(value.translation) },
            rotation: unsafe { std::mem::transmute::<u128, Quat>(value.rotation) },
            scale: Vec3::splat(1.0),
        }
    }
}
#[derive(Component, Default, Debug, Encode, Decode, Eq, PartialEq)]
pub struct SyncObject {
    pub user: u64,
    pub id: u64,
}
#[derive(Component, Default, Debug, Encode, Decode, Copy, Clone)]
#[allow(dead_code)]
pub struct SyncObjectMe(pub u64);
impl SyncObjectMe {
    pub fn new(rand: &mut GlobalEntropy<WyRand>, count: &mut SyncCount) -> Self {
        count.0 += 1;
        Self(rand.next_u64())
    }
}
pub struct SyncCount(pub usize);
impl Resource for SyncCount {}
pub struct Peers(pub Vec<SteamId>);
impl Resource for Peers {}
