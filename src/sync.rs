use crate::download::add_images;
use crate::misc::repaint_face;
use crate::setup::{MAT_HEIGHT, MAT_WIDTH};
use crate::*;
use bevy_steamworks::networking_sockets::{
    ListenSocket, NetConnection, NetPollGroup, NetworkingSockets,
};
use bevy_steamworks::networking_types::{
    ListenSocketEvent, NetworkingConnectionState, NetworkingIdentity, SendFlags,
};
use bevy_steamworks::{
    CallbackResult, ChatMemberStateChange, Client, GameLobbyJoinRequested, LobbyChatUpdate,
    LobbyId, LobbyType, Matchmaking, SteamId, SteamworksEvent,
};
use bitcode::{Decode, Encode, decode, encode};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::mem;
use tokio::sync::mpsc::{Receiver, Sender};
pub fn get_sync(
    client: Res<Client>,
    query: Query<(&SyncObjectMe, &GlobalTransform, Option<&InHand>)>,
    count: Res<SyncCount>,
    mut peers: ResMut<Peers>,
    mut killed: ResMut<Killed>,
) {
    if let Some(id) = peers.lobby_id {
        println!("{:?}", client.matchmaking().lobby_members(id));
        println!("{:?}", peers.list.keys().collect::<Vec<_>>());
    }
    let mut v = Vec::with_capacity(count.0);
    for (id, transform, in_hand) in query {
        v.push((*id, Trans::from(transform), in_hand.is_some()))
    }
    let packet = Packet::Pos(v);
    let bytes = encode(&packet);
    let socket = client.networking_sockets();
    for con in peers.list.values_mut() {
        con.poll(&socket);
        con.send_message(&bytes, SendFlags::RELIABLE);
    }
    for dead in killed.0.drain(..) {
        let packet = Packet::Dead(dead);
        let bytes = encode(&packet);
        for con in peers.list.values() {
            con.send_message(&bytes, SendFlags::RELIABLE);
        }
    }
}
pub fn apply_sync(
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
    println!("a");
    for packet in poll.poll.receive_messages(1024) {
        println!("b");
        let sender = packet.identity_peer().steam_id().unwrap();
        let con = peers.list.get(&sender).unwrap();
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
                        con.send_message(&bytes, SendFlags::RELIABLE);
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
                        con.send_message(&bytes, SendFlags::RELIABLE);
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
                println!("{:?} {:?}", user_changed, member_state_change);
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
                println!("{:?}", lobby_steam_id);
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
    let listen = poll_group.listen.lock().unwrap();
    while let Some(event) = listen.try_receive_event() {
        match event {
            ListenSocketEvent::Connecting(event) => {
                event.accept().unwrap();
            }
            ListenSocketEvent::Connected(event) => {
                let id = event.remote().steam_id().unwrap();
                let connection = event.take_connection();
                connection.set_poll_group(&poll_group.poll);
                peers.list.insert(id, Connection::Waiting(connection));
            }
            ListenSocketEvent::Disconnected(event) => {
                peers.list.remove(&event.remote().steam_id().unwrap());
            }
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
pub fn new_lobby(
    input: Res<ButtonInput<KeyCode>>,
    client: Res<Client>,
    create: Res<LobbyCreateChannel>,
    down: Res<Download>,
) {
    if input.all_pressed([KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::ControlLeft])
        && input.just_pressed(KeyCode::KeyN)
    {
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
        peers.lobby_id = Some(event);
    }
}
pub fn on_create_lobby(mut create: ResMut<LobbyCreateChannel>, mut peers: ResMut<Peers>) {
    while let Ok(event) = create.receiver.try_recv() {
        peers.lobby_id = Some(event);
    }
}
fn connect(peers: &mut Peers, client: &Client, poll_group: &PollGroup, peer: SteamId) {
    let peer_identity = NetworkingIdentity::new_steam_id(peer);
    let connection = client
        .networking_sockets()
        .connect_p2p(peer_identity, 0, None)
        .unwrap();
    connection.set_poll_group(&poll_group.poll);
    peers.list.insert(peer, Connection::Waiting(connection));
}
#[derive(Resource)]
pub struct LobbyJoinChannel {
    pub sender: Sender<LobbyId>,
    pub receiver: Receiver<LobbyId>,
}
#[derive(Resource)]
pub struct LobbyCreateChannel {
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
            translation: unsafe { mem::transmute::<Vec3, (u32, u32, u32)>(value.translation()) },
            rotation: unsafe { mem::transmute::<Quat, u128>(value.rotation()) },
        }
    }
}
impl From<Trans> for Transform {
    fn from(value: Trans) -> Self {
        Self {
            translation: unsafe { mem::transmute::<(u32, u32, u32), Vec3>(value.translation) },
            rotation: unsafe { mem::transmute::<u128, Quat>(value.rotation) },
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
    pub list: HashMap<SteamId, Connection>,
    pub me: usize,
    pub lobby_id: Option<LobbyId>,
}
pub enum Connection {
    Connected(NetConnection),
    Waiting(NetConnection),
    Temp,
}
impl Connection {
    pub fn send_message(&self, data: &[u8], send: SendFlags) {
        if let Connection::Connected(con) = self {
            con.send_message(data, send).unwrap();
        }
    }
    pub fn poll(&mut self, socket: &NetworkingSockets) {
        if let Connection::Waiting(con) = self {
            println!("AAAAAAAAFDFSDFFFFFFFFFFF");
            let info = socket.get_connection_info(con).unwrap();
            println!("{info:?} {:?}", info.state().unwrap());
            if info.state().unwrap() == NetworkingConnectionState::Connected {
                let Connection::Waiting(current) = mem::replace(self, Connection::Temp) else {
                    return;
                };
                *self = Connection::Connected(current);
            }
        }
    }
}
#[derive(Component)]
pub struct InOtherHand;
#[derive(Resource)]
pub struct PollGroup {
    pub poll: NetPollGroup,
    pub listen: Mutex<ListenSocket>,
}
