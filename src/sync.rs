use crate::download::add_images;
use crate::misc::repaint_face;
use crate::setup::{MAT_HEIGHT, MAT_WIDTH};
use crate::*;
use bevy_steamworks::networking_sockets::{NetConnection, NetPollGroup};
use bevy_steamworks::networking_types::{NetworkingIdentity, SendFlags};
use bevy_steamworks::{
    CallbackResult, ChatMemberStateChange, Client, GameLobbyJoinRequested, LobbyChatUpdate,
    LobbyId, LobbyType, Matchmaking, SendType, SteamId, SteamworksEvent,
};
use bitcode::{Decode, Encode, decode, encode};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use tokio::sync::mpsc::{Receiver, Sender};
pub fn get_sync(
    query: Query<(&SyncObjectMe, &GlobalTransform, Option<&InHand>)>,
    count: Res<SyncCount>,
    peers: Res<Peers>,
    mut killed: ResMut<Killed>,
) {
    let mut v = Vec::with_capacity(count.0);
    for (id, transform, in_hand) in query {
        v.push((*id, Trans::from(transform), in_hand.is_some()))
    }
    let packet = Packet::Pos(v);
    let bytes = encode(&packet);
    for con in peers.list.values() {
        con.send_message(&bytes, SendFlags::RELIABLE).unwrap();
    }
    for dead in killed.0.drain(..) {
        let packet = Packet::Dead(dead);
        let bytes = encode(&packet);
        for con in peers.list.values() {
            con.send_message(&bytes, SendFlags::RELIABLE).unwrap();
        }
    }
}
pub fn apply_sync(
    client: Res<Client>,
    mut query: Query<(
        &SyncObject,
        &mut Transform,
        Entity,
        Option<&InOtherHand>,
        &Children,
        Option<&Pile>,
        &mut GravityScale,
    )>,
    mut queryme: Query<(&SyncObjectMe, &GlobalTransform, &Pile), Without<SyncObject>>,
    mut sent: ResMut<Sent>,
    asset_server: Res<AssetServer>,
    down: Res<Download>,
    mut peers: ResMut<Peers>,
    mut commands: Commands,
    hand: Single<Entity, (With<Owned>, With<Hand>)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut poll: ResMut<PollGroup>,
) {
    let networking = client.networking();
    for packet in poll.0.receive_messages(1024) {
        let sender = packet.identity_peer().steam_id().unwrap();
        let data = packet.data();
        let event = decode(data).unwrap();
        match event {
            Packet::Pos(data) => {
                let user = sender.raw();
                for (lid, trans, in_hand) in data {
                    let id = SyncObject { user, id: lid.0 };
                    if let Some((mut t, hand, children, entity, pile, mut gravity)) =
                        query.iter_mut().find_map(|(a, b, e, h, c, p, g)| {
                            if *a == id {
                                Some((b, h, c, e, p, g))
                            } else {
                                None
                            }
                        })
                    {
                        *t = trans.into();
                        if let Some(pile) = pile
                            && in_hand != hand.is_some()
                        {
                            if in_hand {
                                commands.entity(entity).insert(InOtherHand);
                                mats.get_mut(*children.first().unwrap()).unwrap().0 = mats
                                    .get_mut(*children.get(1).unwrap())
                                    .unwrap()
                                    .0
                                    .clone_weak();
                                gravity.0 = 0.0
                            } else {
                                commands.entity(entity).remove::<InOtherHand>();
                                repaint_face(&mut mats, &mut materials, &pile.0[0], children);
                                gravity.0 = GRAVITY;
                            }
                        }
                    } else if sent.0.insert(id) {
                        let bytes = encode(&Packet::Request(lid));
                        networking.send_p2p_packet(sender, SendType::Reliable, &bytes);
                    }
                }
            }
            Packet::Request(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if sent.0.insert(id) {
                    if let Some((b, c)) = queryme
                        .iter_mut()
                        .find_map(|(a, b, c)| if a.0 == lid.0 { Some((b, c)) } else { None })
                    {
                        let bytes = encode(&Packet::New(lid, c.clone_no_image(), Trans::from(b)));
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
            }
            Packet::SetUser => {
                peers.me = peers.list.len();
                info!("joined as number {} user", peers.me);
                commands.entity(*hand).despawn();
                spawn_hand(peers.me, &mut commands);
            }
            Packet::Dead(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some(e) = query
                    .iter_mut()
                    .find_map(|(a, _, b, _, _, _, _)| if *a == id { Some(b) } else { None })
                {
                    commands.entity(e).despawn();
                }
            }
        }
    }
}
pub fn callbacks(
    mut callback: EventReader<SteamworksEvent>,
    client: Res<Client>,
    down: Res<Download>,
    join: Res<LobbyJoinChannel>,
    mut peers: ResMut<Peers>,
    poll_group: Res<PollGroup>,
) {
    for event in callback.read() {
        let SteamworksEvent::CallbackResult(event) = event;
        match event {
            CallbackResult::LobbyChatUpdate(LobbyChatUpdate {
                user_changed,
                member_state_change,
                ..
            }) => {
                if *member_state_change == ChatMemberStateChange::Entered {
                    connect(&mut peers, &client, &poll_group, *user_changed);
                } else {
                    peers.list.remove(user_changed);
                }
            }
            CallbackResult::GameLobbyJoinRequested(GameLobbyJoinRequested {
                lobby_steam_id,
                ..
            }) => {
                let send = join.sender.clone();
                join_lobby(
                    *lobby_steam_id,
                    down.runtime.0.handle().clone(),
                    client.matchmaking(),
                    send,
                )
            }
            _ => {}
        }
    }
}
pub fn spawn_hand(me: usize, commands: &mut Commands) {
    let mut transform = match me {
        0 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        1 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        2 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        3 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        _ => Transform::from_xyz(0.0, 64.0, 0.0),
    };
    if me == 1 || me == 3 {
        transform.rotate_y(PI);
    }
    commands.spawn((transform, Hand::default(), Owned));
}
pub fn new_lobby(input: Res<ButtonInput<KeyCode>>, client: Res<Client>) {
    if input.all_pressed([KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::ControlLeft])
        && input.just_pressed(KeyCode::KeyN)
    {
        client
            .matchmaking()
            .create_lobby(LobbyType::FriendsOnly, 250, |_| {});
    }
}
fn join_lobby(
    id: LobbyId,
    down: tokio::runtime::Handle,
    client: Matchmaking,
    join: Sender<LobbyId>,
) {
    client.join_lobby(id, move |id| {
        if let Ok(id) = id {
            down.spawn(async move {
                let _ = join.send(id).await;
            });
        }
    })
}
pub fn on_join_lobby(
    mut join: ResMut<LobbyJoinChannel>,
    client: Res<Client>,
    mut peers: ResMut<Peers>,
    poll_group: Res<PollGroup>,
) {
    while let Ok(event) = join.receiver.try_recv() {
        let owner = client.matchmaking().lobby_owner(event);
        connect(&mut peers, &client, &poll_group, owner);
    }
}
fn connect(peers: &mut Peers, client: &Client, poll_group: &PollGroup, peer: SteamId) {
    let peer_identity = NetworkingIdentity::new_steam_id(peer);
    let connection = client
        .networking_sockets()
        .connect_p2p(peer_identity, 0, None)
        .unwrap();
    connection.set_poll_group(&poll_group.0);
    peers.list.insert(peer, connection);
}
#[derive(Resource)]
pub struct LobbyJoinChannel {
    pub sender: Sender<LobbyId>,
    pub receiver: Receiver<LobbyId>,
}
#[derive(Resource, Default)]
pub struct Sent(pub HashSet<SyncObject>);
#[derive(Resource, Default)]
pub struct Killed(pub Vec<SyncObjectMe>);
#[derive(Encode, Decode, Debug)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans, bool)>),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    Dead(SyncObjectMe),
    New(SyncObjectMe, Pile, Trans),
    SetUser,
}
#[derive(Encode, Decode, Debug)]
pub struct Trans {
    pub translation: (u32, u32, u32),
    pub rotation: u128,
}
impl Trans {
    fn from(value: &GlobalTransform) -> Self {
        Self {
            translation: unsafe {
                std::mem::transmute::<Vec3, (u32, u32, u32)>(value.translation())
            },
            rotation: unsafe { std::mem::transmute::<Quat, u128>(value.rotation()) },
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
    pub list: HashMap<SteamId, NetConnection>,
    pub me: usize,
}
#[derive(Component)]
pub struct InOtherHand;
#[derive(Resource)]
pub struct PollGroup(pub NetPollGroup);
