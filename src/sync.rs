use crate::download::add_images;
use crate::setup::{MAT_HEIGHT, MAT_WIDTH};
use crate::*;
use bevy_steamworks::{Client, LobbyId, LobbyType, SendType, SteamId};
use bitcode::{Decode, Encode, decode, encode};
use std::collections::HashSet;
use std::f32::consts::PI;
use tokio::sync::mpsc::{Receiver, Sender};
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
    for peer in &peers.list {
        client
            .networking()
            .send_p2p_packet(*peer, SendType::Reliable, &bytes);
    }
}
pub fn apply_sync(
    client: Res<Client>,
    mut query: Query<(&SyncObject, &mut Transform, &Pile)>,
    mut sent: ResMut<Sent>,
    asset_server: Res<AssetServer>,
    down: Res<Download>,
    mut peers: ResMut<Peers>,
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
                    let deck = down.get_deck.clone();
                    let client = down.client.0.clone();
                    let asset_server = asset_server.clone();
                    down.runtime.0.spawn(async move {
                        add_images(pile, trans.into(), id, deck, client, asset_server).await
                    });
                    networking.send_p2p_packet(
                        sender,
                        SendType::Reliable,
                        &encode(&Packet::Received(lid)),
                    );
                }
                Packet::Joined => {
                    for peer in &peers.list {
                        client.networking().send_p2p_packet(
                            *peer,
                            SendType::Reliable,
                            &encode(&Packet::UserJoined(sender.raw())),
                        );
                        client.networking().send_p2p_packet(
                            sender,
                            SendType::Reliable,
                            &encode(&Packet::UserJoined(peer.raw())),
                        );
                    }
                    client.networking().send_p2p_packet(
                        sender,
                        SendType::Reliable,
                        &encode(&Packet::SetUser),
                    );
                    peers.list.push(sender)
                }
                Packet::UserJoined(id) => {
                    peers.list.push(SteamId::from_raw(id));
                }
                Packet::SetUser => {
                    peers.me = peers.list.len();
                    spawn_hand(peers.me, &mut commands);
                }
            }
        }
    }
}
pub fn spawn_hand(me: usize, commands: &mut Commands) {
    let mut transform = match me {
        0 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        1 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        2 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        3 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        _ => Transform::from_xyz(0.0, 64.0, 0.0),
    };
    if me == 2 || me == 3 {
        transform.rotate_y(PI);
    }
    commands.spawn((transform, Hand::default(), Owned));
}
pub fn new_lobby(
    input: Res<ButtonInput<KeyCode>>,
    client: Res<Client>,
    mut clipboard: ResMut<Clipboard>,
    create: Res<LobbyCreateChannel>,
    join: Res<LobbyJoinChannel>,
    down: Res<Download>,
) {
    if input.all_pressed([KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::ControlLeft]) {
        if input.just_pressed(KeyCode::KeyN) {
            let send = create.sender.clone();
            let handle = down.runtime.0.handle().clone();
            client
                .matchmaking()
                .create_lobby(LobbyType::FriendsOnly, 250, move |id| {
                    if let Ok(id) = id {
                        handle.spawn(async move {
                            let _ = send.send(id).await;
                        });
                    }
                });
        } else if input.just_pressed(KeyCode::KeyM)
            && let Ok(id) = clipboard.get_text().parse()
        {
            let send = join.sender.clone();
            let handle = down.runtime.0.handle().clone();
            client
                .matchmaking()
                .join_lobby(LobbyId::from_raw(id), move |id| {
                    if let Ok(id) = id {
                        handle.spawn(async move {
                            let _ = send.send(id).await;
                        });
                    }
                })
        }
    }
}
pub fn on_create_lobby(
    mut create: ResMut<LobbyCreateChannel>,
    mut clipboard: ResMut<Clipboard>,
    mut commands: Commands,
) {
    while let Ok(event) = create.receiver.try_recv() {
        clipboard.0.set_text(event.raw().to_string()).unwrap();
        spawn_hand(0, &mut commands)
    }
}
pub fn on_join_lobby(
    mut join: ResMut<LobbyJoinChannel>,
    client: Res<Client>,
    mut peers: ResMut<Peers>,
) {
    while let Ok(event) = join.receiver.try_recv() {
        let owner = client.matchmaking().lobby_owner(event);
        client
            .networking()
            .send_p2p_packet(owner, SendType::Reliable, &encode(&Packet::Joined));
        peers.list.push(owner);
    }
}
#[derive(Resource)]
pub struct LobbyCreateChannel {
    pub sender: Sender<LobbyId>,
    pub receiver: Receiver<LobbyId>,
}
#[derive(Resource)]
pub struct LobbyJoinChannel {
    pub sender: Sender<LobbyId>,
    pub receiver: Receiver<LobbyId>,
}
#[derive(Resource, Default)]
pub struct Sent(HashSet<SyncObject>);
#[derive(Encode, Decode)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans)>),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    New(SyncObjectMe, Pile, Trans),
    UserJoined(u64),
    SetUser,
    Joined,
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
pub struct Peers {
    pub list: Vec<SteamId>,
    pub me: usize,
}
