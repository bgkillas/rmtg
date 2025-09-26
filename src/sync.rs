use crate::download::add_images;
use crate::misc::{get_mut_card, new_pile_at, repaint_face};
use crate::setup::{MAT_HEIGHT, MAT_WIDTH, spawn_cube, spawn_ico};
use crate::*;
use bitcode::{Decode, Encode, decode, encode};
use net::Reliability;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::mem;
pub fn get_sync(
    query: Query<(
        &SyncObjectMe,
        &GlobalTransform,
        &LinearVelocity,
        &AngularVelocity,
        Option<&InHand>,
    )>,
    mut count: ResMut<SyncCount>,
    mut sync_actions: ResMut<SyncActions>,
    mut sent: ResMut<Sent>,
    query_take: Query<(Entity, &SyncObject)>,
    mut commands: Commands,
    mut client: ResMut<Client>,
) {
    let mut v = Vec::with_capacity(count.0);
    for (id, transform, vel, ang, in_hand) in query {
        v.push((
            *id,
            Trans::from(transform),
            Phys::from(vel, ang),
            in_hand.is_some(),
        ))
    }
    let packet = Packet::Pos(v);
    let bytes = encode(&packet);
    for dead in sync_actions.killed.drain(..) {
        let packet = Packet::Dead(dead);
        let bytes = encode(&packet);
        client.broadcast(&bytes, Reliability::Reliable).unwrap();
    }
    for flip in sync_actions.flip.drain(..) {
        let packet = Packet::Flip(flip);
        let bytes = encode(&packet);
        client.broadcast(&bytes, Reliability::Reliable).unwrap();
    }
    for (from, to) in sync_actions.take_owner.drain(..) {
        if let Some((entity, _)) = query_take.iter().find(|(_, b)| **b == from) {
            commands.entity(entity).remove::<SyncObject>().insert(to);
            count.0 += 1;
        }
        sent.add(from);
        let packet = Packet::Take(from, to);
        let bytes = encode(&packet);
        client.broadcast(&bytes, Reliability::Reliable).unwrap();
    }
    client.broadcast(&bytes, Reliability::Reliable).unwrap();
    #[cfg(feature = "steam")]
    client.flush();
}
pub fn apply_sync(
    mut query: Query<(
        &mut SyncObject,
        &mut Transform,
        Option<&mut Position>,
        Option<&mut Rotation>,
        &mut LinearVelocity,
        &mut AngularVelocity,
        Entity,
        Option<&InOtherHand>,
        Option<&Children>,
        Option<&mut Pile>,
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
    mut commands: Commands,
    hand: Single<Entity, (With<Owned>, With<Hand>)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    card_base: Res<CardBase>,
    mut count: ResMut<SyncCount>,
    mut client: ResMut<Client>,
) {
    let mut ignore = HashSet::new();
    client.recv(|client, packet| {
        let sender = packet.src;
        let data = packet.data;
        let event = decode(&data).unwrap();
        match event {
            Packet::Pos(data) => {
                let user = sender.raw();
                for (lid, trans, phys, in_hand) in data {
                    let id = SyncObject { user, id: lid.0 };
                    if sent.has(&id) || ignore.contains(&id) {
                        continue;
                    }
                    if let Some((
                        mut t,
                        p,
                        r,
                        hand,
                        children,
                        entity,
                        pile,
                        mut gravity,
                        mut lv,
                        mut av,
                    )) = query
                        .iter_mut()
                        .find_map(|(a, b, z, r, lv, av, e, h, c, p, g)| {
                            if *a == id {
                                Some((b, z, r, h, c, e, p, g, lv, av))
                            } else {
                                None
                            }
                        })
                    {
                        let (lvt, avt) = phys.to();
                        *lv = lvt;
                        *av = avt;
                        *t = trans.into();
                        if let Some(mut p) = p {
                            *p = t.translation.into();
                        }
                        if let Some(mut r) = r {
                            *r = t.rotation.into();
                        }
                        if let Some(pile) = pile
                            && in_hand != hand.is_some()
                            && let Some(children) = children
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
                    } else if sent.add(id) {
                        let bytes = encode(&Packet::Request(lid));
                        client
                            .send_message(sender, &bytes, Reliability::Reliable)
                            .unwrap();
                    }
                }
            }
            Packet::Take(from, to) => {
                let new = SyncObject {
                    user: sender.raw(),
                    id: to.0,
                };
                ignore.insert(new);
                if from.user == client.my_id().raw() {
                    let bytes = encode(&Packet::Received(SyncObjectMe(from.id)));
                    client.broadcast(&bytes, Reliability::Reliable).unwrap();
                    if let Some((_, _, _, _, e)) =
                        queryme.iter().find(|(id, _, _, _, _)| id.0 == from.id)
                    {
                        count.0 -= 1;
                        commands
                            .entity(e)
                            .remove::<SyncObjectMe>()
                            .remove::<FollowMouse>()
                            .insert(new);
                    }
                } else if let Some((mut id, _, _, _, _, _, _, _, _, _, _)) = query
                    .iter_mut()
                    .find(|(id, _, _, _, _, _, _, _, _, _, _)| *id.as_ref() == from)
                {
                    sent.add(*id);
                    *id = new
                }
            }
            Packet::Request(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if sent.add(id) {
                    if let Some((b, c, s)) = queryme.iter_mut().find_map(|(a, b, c, s, _)| {
                        if a.0 == lid.0 { Some((b, c, s)) } else { None }
                    }) {
                        if let Some(c) = c {
                            let bytes =
                                encode(&Packet::New(lid, c.clone_no_image(), Trans::from(b)));
                            client
                                .send_message(sender, &bytes, Reliability::Reliable)
                                .unwrap();
                        } else if let Some(s) = s {
                            let bytes = encode(&Packet::NewShape(lid, *s, Trans::from(b)));
                            client
                                .send_message(sender, &bytes, Reliability::Reliable)
                                .unwrap();
                        }
                    } else {
                        sent.rem(&id);
                    }
                }
            }
            Packet::Received(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                sent.del(id);
            }
            Packet::New(lid, pile, trans) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                let deck = down.get_deck.clone();
                let client = down.client.0.clone();
                let asset_server = asset_server.clone();
                let f = async move {
                    add_images(pile, trans.into(), id, deck, client, asset_server).await;
                };
                #[cfg(feature = "wasm")]
                wasm_bindgen_futures::spawn_local(f);
                #[cfg(not(feature = "wasm"))]
                down.runtime.0.spawn(f);
            }
            Packet::NewShape(lid, shape, trans) => {
                let bytes = encode(&Packet::Received(lid));
                client
                    .send_message(sender, &bytes, Reliability::Reliable)
                    .unwrap();
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                sent.del(id);
                ignore.insert(id);
                match shape {
                    Shape::Cube => {
                        let mut e = spawn_cube(
                            256.0,
                            trans.into(),
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                        );
                        e.insert(id);
                    }
                    Shape::Icosahedron => {
                        let mut e = spawn_ico(
                            64.0,
                            trans.into(),
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                        );
                        e.insert(id);
                    }
                }
            }
            Packet::SetUser(id) => {
                info!("joined as number {} user", id);
                commands.entity(*hand).despawn();
                spawn_hand(id, &mut commands);
            }
            Packet::Dead(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some(e) = query.iter_mut().find_map(
                    |(a, _, _, _, _, _, b, _, _, _, _)| if *a == id { Some(b) } else { None },
                ) {
                    commands.entity(e).despawn();
                }
            }
            Packet::Flip(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some((transform, children, mut pile)) = query.iter_mut().find_map(
                    |(a, b, _, _, _, _, _, _, c, d, _)| {
                        if *a == id { Some((b, c, d)) } else { None }
                    },
                ) && let Some(pile) = &mut pile
                {
                    let card = get_mut_card(pile, &transform);
                    if let Some(alt) = &mut card.alt
                        && let Some(children) = children
                    {
                        mem::swap(&mut card.normal, alt);
                        repaint_face(&mut mats, &mut materials, card, children);
                        card.is_alt = !card.is_alt;
                    }
                }
            }
            Packet::Reorder(lid, order) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some(mut pile) = query.iter_mut().find_map(
                    |(a, _, _, _, _, _, _, _, _, d, _)| {
                        if *a == id { Some(d) } else { None }
                    },
                ) && let Some(pile) = &mut pile
                {
                    for (i, id) in order.into_iter().enumerate() {
                        let n = pile.0[i..].iter().position(|c| c.id == id).unwrap() + i;
                        pile.0.swap(i, n);
                    }
                }
            }
            Packet::Draw(lid, to, start) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some(mut pile) = query.iter_mut().find_map(
                    |(a, _, _, _, _, _, _, _, _, d, _)| {
                        if *a == id { Some(d) } else { None }
                    },
                ) && let Some(pile) = &mut pile
                {
                    let len = to.len();
                    for ((id, trans), card) in to.into_iter().zip(pile.0.drain(start - len..start))
                    {
                        new_pile_at(
                            Pile(vec![card]),
                            card_base.stock.clone_weak(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            card_base.back.clone_weak(),
                            card_base.side.clone_weak(),
                            trans.into(),
                            false,
                            None,
                            Some(SyncObject { user, id: id.0 }),
                            None,
                        );
                    }
                }
            }
        }
    });
}
pub fn spawn_hand(me: usize, commands: &mut Commands) {
    let mut transform = match me {
        0 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        1 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        2 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        3 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        _ => Transform::from_xyz(0.0, 64.0, 0.0),
    };
    if me == 1 || me == 3 {
        transform.rotate_y(PI);
    }
    commands.spawn((transform, Hand::default(), Owned));
}
#[cfg(all(feature = "steam", feature = "ip"))]
pub fn new_lobby(
    input: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<Client>,
    down: Res<Download>,
) {
    if input.all_pressed([KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::ControlLeft]) {
        if input.just_pressed(KeyCode::KeyN) {
            info!("hosting steam");
            #[cfg(feature = "steam")]
            client.host_steam().unwrap();
        } else if input.just_pressed(KeyCode::KeyM) {
            info!("hosting ip");
            #[cfg(feature = "ip")]
            client
                .host_ip_runtime(
                    Some(Box::new(|client, peer| {
                        client
                            .send_message(
                                peer,
                                &encode(&Packet::SetUser(peer.0 as usize)),
                                Reliability::Reliable,
                            )
                            .unwrap();
                    })),
                    None,
                    &down.runtime.0,
                )
                .unwrap();
        } else if input.just_pressed(KeyCode::KeyK) {
            info!("joining ip");
            #[cfg(feature = "ip")]
            client
                .join_ip_runtime("127.0.0.1".parse().unwrap(), None, None, &down.runtime.0)
                .unwrap();
        }
    }
}
#[derive(Resource, Default)]
pub struct Sent(pub HashMap<SyncObject, bool>);
impl Sent {
    pub fn add(&mut self, key: SyncObject) -> bool {
        if let Some(b) = self.0.insert(key, true) {
            if b {
                false
            } else {
                self.0.remove(&key);
                true
            }
        } else {
            true
        }
    }
    pub fn del(&mut self, key: SyncObject) -> bool {
        if self.0.remove(&key).is_some() {
            true
        } else {
            self.0.insert(key, false);
            false
        }
    }
    pub fn rem(&mut self, key: &SyncObject) {
        self.0.remove(key);
    }
    pub fn has(&self, key: &SyncObject) -> bool {
        self.0.get(key) == Some(&true)
    }
}
#[derive(Resource, Default)]
pub struct SyncActions {
    pub killed: Vec<SyncObjectMe>,
    pub take_owner: Vec<(SyncObject, SyncObjectMe)>,
    pub reorder: Vec<(SyncObjectMe, Vec<String>)>, //TODO
    pub draw: Vec<(SyncObjectMe, Pile, usize)>,
    pub flip: Vec<SyncObjectMe>,
}
#[derive(Encode, Decode, Debug)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans, Phys, bool)>),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    Dead(SyncObjectMe),
    Take(SyncObject, SyncObjectMe),
    New(SyncObjectMe, Pile, Trans),
    NewShape(SyncObjectMe, Shape, Trans),
    Flip(SyncObjectMe),
    Reorder(SyncObjectMe, Vec<String>),
    Draw(SyncObjectMe, Vec<(SyncObjectMe, Trans)>, usize),
    SetUser(usize),
}
#[derive(Encode, Decode, Debug)]
pub struct Phys {
    pub vel: (u32, u32, u32),
    pub ang: (u32, u32, u32),
}
impl Phys {
    fn from(pos: &LinearVelocity, ang: &AngularVelocity) -> Self {
        Self {
            vel: unsafe { mem::transmute::<LinearVelocity, (u32, u32, u32)>(*pos) },
            ang: unsafe { mem::transmute::<AngularVelocity, (u32, u32, u32)>(*ang) },
        }
    }
    fn to(self) -> (LinearVelocity, AngularVelocity) {
        unsafe { mem::transmute::<Self, (LinearVelocity, AngularVelocity)>(self) }
    }
}
#[derive(Encode, Decode, Component, Copy, Clone, Debug)]
pub enum Shape {
    Cube,
    Icosahedron,
}
#[derive(Encode, Decode, Debug, Copy, Clone)]
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
#[derive(Component, Default, Debug, Encode, Decode, Eq, PartialEq, Copy, Clone)]
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
#[derive(Component)]
pub struct InOtherHand;
