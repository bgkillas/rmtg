use crate::misc::new_pile_at;
use crate::*;
use bevy_steamworks::{Client, SendType, SteamId};
use bitcode::{Decode, Encode, decode, encode};
use std::collections::HashSet;
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
    for peer in &peers.0 {
        client
            .networking()
            .send_p2p_packet(*peer, SendType::Reliable, &bytes);
    }
}
pub fn apply_sync(
    client: Res<Client>,
    mut query: Query<(&SyncObject, &mut Transform, &Pile)>,
    mut sent: ResMut<Sent>,
    card_base: Res<CardBase>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let networking = client.networking();
    while let Some(size) = networking.is_p2p_packet_available() {
        let mut data = vec![0; size];
        if let Some((sender, _)) = networking.read_p2p_packet(&mut data) {
            match decode(&data).unwrap() {
                Packet::Pos(data) => {
                    let user = sender.raw();
                    for (lid, trans) in data {
                        let id = SyncObject { user, id: lid.0 };
                        if let Some(mut t) = query
                            .iter_mut()
                            .find_map(|(a, b, _)| if *a == id { Some(b) } else { None })
                        {
                            *t = trans.into()
                        } else {
                            let bytes = encode(&Packet::Request(lid));
                            networking.send_p2p_packet(sender, SendType::Reliable, &bytes);
                        }
                    }
                }
                Packet::Request(lid) => {
                    let user = sender.raw();
                    let id = SyncObject { user, id: lid.0 };
                    if sent.0.insert(id) {
                        if let Some((b, c)) = query
                            .iter_mut()
                            .find_map(|(a, b, c)| if *a == id { Some((b, c)) } else { None })
                        {
                            let bytes =
                                encode(&Packet::New(lid, c.clone_no_image(), Trans::from(&b)));
                            networking.send_p2p_packet(sender, SendType::Reliable, &bytes);
                        } else {
                            sent.0.remove(&id);
                        }
                    }
                }
                Packet::Received(lid) => {
                    let user = sender.raw();
                    let id = SyncObject { user, id: lid.0 };
                    sent.0.remove(&id);
                }
                Packet::New(lid, pile, trans) => {
                    let user = sender.raw();
                    let id = SyncObject { user, id: lid.0 };
                    let ent = new_pile_at(
                        pile,
                        card_base.stock.clone_weak(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        card_base.back.clone_weak(),
                        card_base.side.clone_weak(),
                        trans.into(),
                        None,
                        false,
                        false,
                        None,
                        None,
                    );
                    commands.entity(ent.unwrap()).insert(id);
                    networking.send_p2p_packet(
                        sender,
                        SendType::Reliable,
                        &encode(&Packet::Received(lid)),
                    );
                }
            }
        }
    }
}
#[derive(Resource, Default)]
pub struct Sent(HashSet<SyncObject>);
#[derive(Encode, Decode)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans)>),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    New(SyncObjectMe, Pile, Trans),
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
#[derive(Component, Default, Debug, Encode, Decode, Eq, PartialEq, Hash, Copy, Clone)]
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
#[derive(Resource, Default)]
pub struct SyncCount(pub usize);
#[derive(Resource, Default)]
pub struct Peers(pub Vec<SteamId>);
