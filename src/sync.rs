use crate::download::add_images;
use crate::misc::repaint_face;
use crate::setup::{MAT_HEIGHT, MAT_WIDTH, spawn_cube, spawn_ico};
use crate::*;
use bevy_steamworks::networking_sockets::{
    ListenSocket, NetConnection, NetPollGroup, NetworkingSockets,
};
use bevy_steamworks::networking_types::{
    ListenSocketEvent, NetworkingConnectionState, NetworkingIdentity, SendFlags,
};
use bevy_steamworks::{
    CallbackResult, Client, GameLobbyJoinRequested, LobbyId, LobbyType, Matchmaking, SteamId,
    SteamworksEvent,
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
    mut sync_actions: ResMut<SyncActions>,
) {
    let mut v = Vec::with_capacity(count.0);
    for (id, transform, in_hand) in query {
        v.push((*id, Trans::from(transform), in_hand.is_some()))
    }
    let packet = Packet::Pos(v);
    let bytes = encode(&packet);
    let socket = client.networking_sockets();
    for dead in sync_actions.killed.drain(..) {
        let packet = Packet::Dead(dead);
        let bytes = encode(&packet);
        for con in peers.list.values() {
            con.send_message(&bytes, SendFlags::RELIABLE);
        }
    }
    for (from, to) in sync_actions.take_owner.drain(..) {
        let packet = Packet::Take(from, to);
        let bytes = encode(&packet);
        for con in peers.list.values() {
            con.send_message(&bytes, SendFlags::RELIABLE);
        }
    }
    for con in peers.list.values_mut() {
        con.poll(&socket);
        con.send_message(&bytes, SendFlags::RELIABLE);
    }
}
pub fn apply_sync(
    mut query: Query<(
        &mut SyncObject,
        &mut Transform,
        &mut Position,
        &mut Rotation,
        Entity,
        Option<&InOtherHand>,
        &Children,
        Option<&Pile>,
        &mut GravityScale,
    )>,
    mut queryme: Query<
        (
            &SyncObjectMe,
            &GlobalTransform,
            Option<&Pile>,
            Option<&Shape>,
            Entity,
        ),
        Without<SyncObject>,
    >,
    mut sent: ResMut<Sent>,
    asset_server: Res<AssetServer>,
    down: Res<Download>,
    mut peers: ResMut<Peers>,
    mut commands: Commands,
    hand: Single<Entity, (With<Owned>, With<Hand>)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut poll: ResMut<PollGroup>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for packet in poll.poll.receive_messages(1024) {
        let sender = packet.identity_peer().steam_id().unwrap();
        let con = peers.list.get(&sender).unwrap();
        let data = packet.data();
        let event = decode(data).unwrap();
        match event {
            Packet::Pos(data) => {
                let user = sender.raw();
                for (lid, trans, in_hand) in data {
                    let id = SyncObject { user, id: lid.0 };
                    if let Some((mut t, mut p, mut r, hand, children, entity, pile, mut gravity)) =
                        query.iter_mut().find_map(|(a, b, z, r, e, h, c, p, g)| {
                            if *a == id {
                                Some((b, z, r, h, c, e, p, g))
                            } else {
                                None
                            }
                        })
                    {
                        *t = trans.into();
                        *p = t.translation.into();
                        *r = t.rotation.into();
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
            Packet::Take(from, to) => {
                let new = SyncObject {
                    user: sender.raw(),
                    id: to.0,
                };
                if from.user == peers.my_id.raw() {
                    if let Some((_, _, _, _, e)) =
                        queryme.iter().find(|(id, _, _, _, _)| id.0 == from.id)
                    {
                        commands
                            .entity(e)
                            .remove::<SyncObjectMe>()
                            .remove::<FollowMouse>()
                            .insert(new);
                    }
                } else if let Some((mut id, _, _, _, _, _, _, _, _)) = query
                    .iter_mut()
                    .find(|(id, _, _, _, _, _, _, _, _)| *id.as_ref() == from)
                {
                    *id = new
                }
            }
            Packet::Request(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if sent.0.insert(id) {
                    if let Some((b, c, s)) = queryme.iter_mut().find_map(|(a, b, c, s, _)| {
                        if a.0 == lid.0 { Some((b, c, s)) } else { None }
                    }) {
                        if let Some(c) = c {
                            let bytes =
                                encode(&Packet::New(lid, c.clone_no_image(), Trans::from(b)));
                            con.send_message(&bytes, SendFlags::RELIABLE);
                        } else if let Some(s) = s {
                            let bytes = encode(&Packet::NewShape(lid, *s, Trans::from(b)));
                            con.send_message(&bytes, SendFlags::RELIABLE);
                        }
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
            Packet::NewShape(lid, shape, trans) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                match shape {
                    Shape::Cube => {
                        spawn_cube(
                            256.0,
                            trans.into(),
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &asset_server,
                        )
                        .insert(id);
                    }
                    Shape::Icosahedron => {
                        spawn_ico(
                            64.0,
                            trans.into(),
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                        )
                        .insert(id);
                    }
                }
            }
            Packet::SetUser(id) => {
                peers.me = id;
                info!("joined as number {} user", peers.me);
                commands.entity(*hand).despawn();
                spawn_hand(peers.me, &mut commands);
            }
            Packet::Dead(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some(e) = query
                    .iter_mut()
                    .find_map(|(a, _, _, _, b, _, _, _, _)| if *a == id { Some(b) } else { None })
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
        if let CallbackResult::GameLobbyJoinRequested(GameLobbyJoinRequested {
            lobby_steam_id,
            ..
        }) = event
        {
            let send = join.sender.clone();
            join_lobby(
                *lobby_steam_id,
                down.runtime.0.handle().clone(),
                client.matchmaking(),
                send,
            )
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
                info!("connected to {id:?}");
                let connection = event.take_connection();
                connection.set_poll_group(&poll_group.poll);
                if peers.is_host {
                    peers.count += 1;
                    connection
                        .send_message(&encode(&Packet::SetUser(peers.count)), SendFlags::RELIABLE)
                        .unwrap();
                }
                peers.list.insert(id, Connection::Waiting(connection));
            }
            ListenSocketEvent::Disconnected(event) => {
                let id = event.remote().steam_id().unwrap();
                peers.list.remove(&id);
                info!("disconnected from {id:?}");
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
        peers.host_id = owner;
        peers.is_host = false;
        info!("connecting to {owner:?}");
        let my_id = peers.my_id;
        for id in client.matchmaking().lobby_members(event) {
            if id != my_id {
                connect(&mut peers, &client, &poll_group, id);
            }
        }
        peers.lobby_id = event;
    }
}
pub fn on_create_lobby(mut create: ResMut<LobbyCreateChannel>, mut peers: ResMut<Peers>) {
    while let Ok(event) = create.receiver.try_recv() {
        peers.host_id = peers.my_id;
        peers.is_host = true;
        info!("created lobby {event:?}");
        peers.lobby_id = event;
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
pub struct SyncActions {
    pub killed: Vec<SyncObjectMe>,
    pub take_owner: Vec<(SyncObject, SyncObjectMe)>,
}
#[derive(Encode, Decode, Debug)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans, bool)>),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    Dead(SyncObjectMe),
    Take(SyncObject, SyncObjectMe),
    New(SyncObjectMe, Pile, Trans),
    NewShape(SyncObjectMe, Shape, Trans),
    SetUser(usize),
}
#[derive(Encode, Decode, Component, Copy, Clone, Debug)]
pub enum Shape {
    Cube,
    Icosahedron,
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
#[derive(Resource)]
pub struct Peers {
    pub list: HashMap<SteamId, Connection>,
    pub me: usize,
    pub count: usize,
    pub lobby_id: LobbyId,
    pub host_id: SteamId,
    pub my_id: SteamId,
    pub is_host: bool,
}
impl Default for Peers {
    fn default() -> Self {
        Self {
            list: default(),
            me: 0,
            count: 0,
            lobby_id: LobbyId::from_raw(0),
            host_id: SteamId::from_raw(0),
            my_id: SteamId::from_raw(0),
            is_host: false,
        }
    }
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
            let info = socket.get_connection_info(con).unwrap();
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
