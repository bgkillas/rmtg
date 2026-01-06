use crate::setup::MAT_WIDTH;
use crate::shapes::Side;
use crate::sync::{CameraInd, CursorInd, SyncObjectMe};
use crate::*;
use bevy_tangled::PeerId;
use std::f32::consts::PI;
pub fn vec2_to_ground(pile: &Pile, v: Vec2, rev: bool) -> Transform {
    let size = pile.len() as f32 * CARD_THICKNESS;
    let mut transform = Transform::from_xyz(v.x, size / 2.0, v.y);
    if rev {
        transform.rotate_local_z(PI);
    }
    if transform.translation.z.is_sign_negative() {
        rotate_right(&mut transform);
        rotate_right(&mut transform)
    }
    transform
}
pub fn new_pile(
    pile: Pile,
    card_base: CardBase,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    v: Vec2,
    id: Option<SyncObject>,
    my_id: Option<SyncObjectMe>,
    rev: bool,
) -> Option<Entity> {
    let transform = vec2_to_ground(&pile, v, rev);
    new_pile_at(
        pile, card_base, materials, commands, meshes, transform, false, None, id, my_id,
    )
    .map(|a| a.id())
}
fn side(size: f32, meshes: &mut Assets<Mesh>) -> (Handle<Mesh>, Transform, Transform) {
    let mesh = meshes.add(Rectangle::new(size, CARD_HEIGHT));
    (
        mesh,
        Transform::from_xyz(CARD_WIDTH / 2.0, 0.0, 0.0).looking_to(Dir3::NEG_X, Dir3::Z),
        Transform::from_xyz(-CARD_WIDTH / 2.0, 0.0, 0.0).looking_to(Dir3::X, Dir3::Z),
    )
}
fn topbottom(size: f32, meshes: &mut Assets<Mesh>) -> (Handle<Mesh>, Transform, Transform) {
    let mesh2 = meshes.add(Rectangle::new(CARD_WIDTH, size));
    (
        mesh2,
        Transform::from_xyz(0.0, 0.0, -CARD_HEIGHT / 2.0).looking_to(Dir3::Z, Dir3::NEG_Y),
        Transform::from_xyz(0.0, 0.0, CARD_HEIGHT / 2.0).looking_to(Dir3::NEG_Z, Dir3::NEG_Y),
    )
}
pub fn new_pile_at<'a>(
    pile: Pile,
    card_base: CardBase,
    materials: &mut Assets<StandardMaterial>,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    transform: Transform,
    follow_mouse: bool,
    parent: Option<Entity>,
    id: Option<SyncObject>,
    sync_object: Option<SyncObjectMe>,
) -> Option<EntityCommands<'a>> {
    let ent = {
        if pile.is_empty() {
            return None;
        }
        let card = pile.last();
        let size = pile.len() as f32 * CARD_THICKNESS;
        let mut ent = commands.spawn((
            transform,
            Visibility::default(),
            RigidBody::Dynamic,
            LinearDamping(LIN_DAMPING),
            AngularDamping(ANG_DAMPING),
            SLEEP,
            CollisionLayers::new(0b11, LayerMask::ALL),
            Collider::cuboid(CARD_WIDTH, size, CARD_HEIGHT),
            CollisionEventsEnabled,
            GravityScale(if follow_mouse || parent.is_some() {
                0.0
            } else {
                GRAVITY
            }),
            card_bundle(size, card_base.clone(), materials, meshes, card),
        ));
        if let Some(id) = id {
            ent.insert(id);
        } else if let Some(obj) = sync_object {
            ent.insert(obj);
        }
        if follow_mouse {
            ent.insert(FollowMouse);
        }
        if let Some(parent) = parent {
            ent.insert(ChildOf(parent));
        }
        ent.id()
    };
    if pile.is_equiped() {
        spawn_equip(ent, &pile, commands, card_base, materials, meshes);
    }
    let mut ent = commands.entity(ent);
    ent.insert(pile);
    Some(ent)
}
pub fn card_bundle(
    size: f32,
    card_base: CardBase,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    top: &SubCard,
) -> impl Bundle {
    let material_handle = materials.add(top.material());
    let (mesh, transform2, transform3) = side(size, meshes);
    let (mesh2, transform4, transform5) = topbottom(size, meshes);
    let card_stock = card_base.stock;
    let card_side = card_base.side;
    let card_back = card_base.back;
    children![
        (
            Mesh3d(card_stock.clone()),
            MeshMaterial3d(material_handle),
            Transform::from_xyz(0.0, size / 2.0, 0.0).looking_to(Dir3::NEG_Y, Dir3::NEG_Z),
        ),
        (
            Mesh3d(card_stock),
            MeshMaterial3d(card_back),
            Transform::from_xyz(0.0, -size / 2.0, 0.0).looking_to(Dir3::Y, Dir3::NEG_Z)
        ),
        (
            Mesh3d(mesh.clone()),
            MeshMaterial3d(card_side.clone()),
            transform2,
        ),
        (Mesh3d(mesh), MeshMaterial3d(card_side.clone()), transform3),
        (
            Mesh3d(mesh2.clone()),
            MeshMaterial3d(card_side.clone()),
            transform4,
        ),
        (Mesh3d(mesh2), MeshMaterial3d(card_side), transform5)
    ]
}
pub fn is_reversed(transform: &Transform) -> bool {
    (transform.rotation * Vec3::Y).y < 0.0
}
pub fn repaint_face(
    mats: &mut Query<&mut MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    card: &SubCard,
    children: &Children,
) {
    mats.get_mut(*children.first().unwrap()).unwrap().0 = materials.add(card.material());
}
pub fn adjust_meshes(
    pile: &Pile,
    children: &Children,
    meshes: &mut Assets<Mesh>,
    query: &mut Query<
        (&mut Mesh3d, &mut Transform),
        (
            Without<Children>,
            With<ChildOf>,
            Without<InHand>,
            Without<Shape>,
            Without<Pile>,
            Without<Side>,
        ),
    >,
    transform: &mut Transform,
    collider: &mut Collider,
    equipment: &Query<(), Or<(With<Equipment>, With<Counter>)>>,
    commands: &mut Commands,
) {
    let size = pile.len() as f32 * CARD_THICKNESS;
    *collider = Collider::cuboid(CARD_WIDTH, size, CARD_HEIGHT);
    let mut children = children.iter();
    let (_, mut top) = query.get_mut(children.next().unwrap()).unwrap();
    let delta = top.translation.y - size / 2.0;
    top.translation.y -= delta;
    let (_, mut bottom) = query.get_mut(children.next().unwrap()).unwrap();
    bottom.translation.y += delta;
    transform.translation.y -= delta;
    let (mesh, t1, t2) = side(size, meshes);
    let (mut leftmesh, mut leftt) = query.get_mut(children.next().unwrap()).unwrap();
    leftmesh.0 = mesh.clone();
    *leftt = t1;
    let (mut rightmesh, mut rightt) = query.get_mut(children.next().unwrap()).unwrap();
    rightmesh.0 = mesh;
    *rightt = t2;
    let (mesh2, t1, t2) = topbottom(size, meshes);
    let (mut topmesh, mut topt) = query.get_mut(children.next().unwrap()).unwrap();
    topmesh.0 = mesh2.clone();
    *topt = t1;
    let (mut bottommesh, mut bottomt) = query.get_mut(children.next().unwrap()).unwrap();
    bottommesh.0 = mesh2;
    *bottomt = t2;
    for c in children {
        if equipment.contains(c) {
            commands.entity(c).despawn();
        }
    }
}
pub fn spawn_equip(
    ent: Entity,
    pile: &Pile,
    commands: &mut Commands,
    card_base: CardBase,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    commands.entity(ent).with_children(|parent| {
        for (i, c) in pile.iter_equipment().rev().enumerate() {
            let top = i.is_multiple_of(2);
            let transform = Transform::from_xyz(
                (EQUIP_SCALE * ((i & !1) + 1) as f32 + 1.0) * CARD_WIDTH / 2.0,
                0.0,
                if top { -EQUIP_SCALE } else { EQUIP_SCALE } * CARD_HEIGHT / 2.0,
            )
            .with_scale(Vec3::splat(EQUIP_SCALE));
            parent.spawn((
                Equipment,
                transform,
                InheritedVisibility::default(),
                card_bundle(CARD_THICKNESS, card_base.clone(), materials, meshes, c),
            ));
        }
    });
}
#[derive(Component)]
pub struct Equipment;
//TODO
#[derive(Component)]
pub struct Counter;
pub fn make_cam(
    commands: &mut Commands,
    peer: PeerId,
    id: usize,
    pos: Vec3,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(CARD_THICKNESS * 64.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            alpha_mode: AlphaMode::Opaque,
            unlit: true,
            base_color: PLAYER[id % PLAYER.len()],
            ..default()
        })),
        Transform::from_xyz(pos.x, pos.y, pos.z),
        CameraInd(peer),
    ));
}
pub fn make_cur(
    commands: &mut Commands,
    peer: PeerId,
    id: usize,
    pos: Vec3,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(CARD_THICKNESS * 16.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            alpha_mode: AlphaMode::Opaque,
            unlit: true,
            base_color: PLAYER[id % PLAYER.len()],
            ..default()
        })),
        Transform::from_xyz(pos.x, pos.y, pos.z),
        CursorInd(peer, false),
    ));
}
pub fn default_cam_pos(n: usize) -> Transform {
    let x = if n / 2 == 0 {
        MAT_WIDTH / 2.0
    } else {
        -MAT_WIDTH / 2.0
    };
    let z = if n.is_multiple_of(2) {
        MAT_WIDTH
    } else {
        -MAT_WIDTH
    };
    Transform::from_xyz(x, START_Y, z).looking_at(Vec3::new(x, 0.0, 0.0), Vec3::Y)
}
pub fn rotate_left(transform: &mut Transform) {
    let (_, rot, _) = transform.rotation.to_euler(EulerRot::XYZ);
    let n = (2.0 * rot / PI).round() as isize;
    let n = (n + 1).rem_euclid(4);
    transform.rotate_y(n as f32 * (PI / 2.0) - rot);
}
pub fn rotate_right(transform: &mut Transform) {
    let (_, rot, _) = transform.rotation.to_euler(EulerRot::XYZ);
    let n = (2.0 * rot / PI).round() as isize;
    let n = (n - 1).rem_euclid(4);
    transform.rotate_y(n as f32 * (PI / 2.0) - rot);
}
pub fn ui_rotate_right(transform: &mut UiTransform) {
    transform.rotation = match transform.rotation.sin_cos() {
        (0.0, 1.0) => Rot2::from_sin_cos(1.0, 0.0),
        (1.0, 0.0) => Rot2::from_sin_cos(0.0, -1.0),
        (0.0, -1.0) => Rot2::from_sin_cos(-1.0, 0.0),
        (-1.0, 0.0) => Rot2::from_sin_cos(0.0, 1.0),
        _ => unreachable!(),
    };
    transform.translation = if matches!(transform.rotation.sin_cos(), (1.0, 0.0) | (-1.0, 0.0)) {
        Val2::px(
            (IMAGE_HEIGHT - IMAGE_WIDTH) / 2.0,
            (IMAGE_WIDTH - IMAGE_HEIGHT) / 2.0,
        )
    } else {
        Val2::px(0.0, 0.0)
    };
}
pub fn ui_rotate_left(transform: &mut UiTransform) {
    transform.rotation = match transform.rotation.sin_cos() {
        (0.0, 1.0) => Rot2::from_sin_cos(-1.0, 0.0),
        (-1.0, 0.0) => Rot2::from_sin_cos(0.0, -1.0),
        (0.0, -1.0) => Rot2::from_sin_cos(1.0, 0.0),
        (1.0, 0.0) => Rot2::from_sin_cos(0.0, 1.0),
        _ => unreachable!(),
    };
    transform.translation = if matches!(transform.rotation.sin_cos(), (1.0, 0.0) | (-1.0, 0.0)) {
        Val2::px(
            (IMAGE_HEIGHT - IMAGE_WIDTH) / 2.0,
            (IMAGE_WIDTH - IMAGE_HEIGHT) / 2.0,
        )
    } else {
        Val2::px(0.0, 0.0)
    };
}
pub fn remove_follow(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<FollowMouse>()
        .remove::<ShapeHold>();
}
