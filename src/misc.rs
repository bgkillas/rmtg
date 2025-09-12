use crate::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
use std::f32::consts::PI;
pub fn make_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    top: Handle<Image>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(top),
        unlit: true,
        ..default()
    })
}
pub fn new_pile(
    pile: Pile,
    card_stock: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    card_back: Handle<StandardMaterial>,
    card_side: Handle<StandardMaterial>,
    rand: &mut GlobalEntropy<WyRand>,
    x: f32,
    z: f32,
) {
    let size = pile.0.len() as f32;
    let mut transform = Transform::from_xyz(x, size, z);
    transform.rotate_x(-PI / 2.0);
    new_pile_at(
        pile, card_stock, materials, commands, meshes, card_back, card_side, transform, rand,
        false, false, None,
    );
}
pub fn new_pile_at(
    pile: Pile,
    card_stock: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    card_back: Handle<StandardMaterial>,
    card_side: Handle<StandardMaterial>,
    transform: Transform,
    rand: &mut GlobalEntropy<WyRand>,
    follow_mouse: bool,
    reverse: bool,
    parent: Option<Entity>,
) -> Option<Entity> {
    if pile.0.is_empty() {
        return None;
    }
    let card = pile.0.last().unwrap();
    let top = card.normal.image.clone_weak();
    let material_handle = make_material(materials, top);
    let size = pile.0.len() as f32;
    let mut transform1 = Transform::from_rotation(Quat::from_rotation_y(PI));
    transform1.translation.z = -size;
    let mesh = meshes.add(Rectangle::new(2.0 * size, CARD_HEIGHT));
    let mut transform2 = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));
    transform2.translation.x = CARD_WIDTH / 2.0;
    let mut transform3 = Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0));
    transform3.translation.x = -CARD_WIDTH / 2.0;
    let mesh2 = meshes.add(Rectangle::new(CARD_WIDTH, 2.0 * size));
    let mut transform4 = Transform::from_rotation(Quat::from_rotation_x(PI / 2.0));
    transform4.translation.y = -CARD_HEIGHT / 2.0;
    let mut transform5 = Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0));
    transform5.translation.y = CARD_HEIGHT / 2.0;
    let mut ent = commands.spawn((
        pile,
        transform,
        Visibility::default(),
        RigidBody::Dynamic,
        Collider::cuboid(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, size),
        GravityScale(if follow_mouse || parent.is_some() {
            0.0
        } else {
            GRAVITY
        }),
        Ccd::enabled(),
        Velocity::zero(),
        Damping {
            linear_damping: DAMPING,
            angular_damping: 0.0,
        },
        AdditionalMassProperties::Mass(size),
        SyncObject::new(rand),
        children![
            (
                Mesh3d(card_stock.clone_weak()),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(0.0, 0.0, size),
            ),
            (Mesh3d(card_stock), MeshMaterial3d(card_back), transform1),
            (
                Mesh3d(mesh.clone_weak()),
                MeshMaterial3d(card_side.clone_weak()),
                transform2,
            ),
            (
                Mesh3d(mesh),
                MeshMaterial3d(card_side.clone_weak()),
                transform3,
            ),
            (
                Mesh3d(mesh2.clone_weak()),
                MeshMaterial3d(card_side.clone_weak()),
                transform4,
            ),
            (Mesh3d(mesh2), MeshMaterial3d(card_side), transform5)
        ],
    ));
    if follow_mouse {
        ent.insert(FollowMouse);
    }
    if reverse {
        ent.insert(Reversed);
    }
    if let Some(parent) = parent {
        ent.insert(ChildOf(parent));
    }
    Some(ent.id())
}
