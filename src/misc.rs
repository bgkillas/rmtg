use crate::setup::Wall;
use crate::sync::SyncObjectMe;
use crate::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use std::f32::consts::PI;
pub fn make_material(
    materials: &mut Assets<StandardMaterial>,
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
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    card_back: Handle<StandardMaterial>,
    card_side: Handle<StandardMaterial>,
    rand: &mut Single<&mut WyRand, With<GlobalRng>>,
    v: Vec2,
    count: &mut SyncCount,
    id: Option<SyncObject>,
) -> Option<Entity> {
    let size = pile.len() as f32;
    let mut transform = Transform::from_xyz(v.x, size, v.y);
    transform.rotate_x(-PI / 2.0);
    new_pile_at(
        pile,
        card_stock,
        materials,
        commands,
        meshes,
        card_back,
        card_side,
        transform,
        false,
        None,
        id,
        if id.is_none() {
            Some(SyncObjectMe::new(rand, count))
        } else {
            None
        },
    )
    .map(|a| a.id())
}
pub fn move_up(
    entity: Entity,
    ents: &Query<(&Collider, &mut Transform), Without<Wall>>,
    pset: &mut ParamSet<(Query<&mut Position>, SpatialQuery)>,
) {
    let mut excluded = vec![entity];
    let (collider, transform) = ents.get(entity).unwrap();
    let rotation = transform.rotation;
    let mut translation = transform.translation;
    while let Some(m) = pset
        .p1()
        .shape_intersections(
            collider,
            translation,
            rotation,
            &SpatialQueryFilter::DEFAULT.with_excluded_entities(excluded.clone()),
        )
        .into_iter()
        .filter_map(|a| {
            excluded.push(a);
            if let Ok((collider, transform)) = ents.get(a) {
                let y = collider
                    .aabb(transform.translation, transform.rotation)
                    .max
                    .y;
                Some(y)
            } else {
                None
            }
        })
        .reduce(f32::max)
    {
        translation.y = m;
        let (collider, transform) = ents.get(entity).unwrap();
        let aabb = collider.aabb(transform.translation, transform.rotation);
        let max = m + (aabb.max.y - aabb.min.y) / 2.0 + 4.0;
        let max = max.max(aabb.max.y);
        let mut pos = pset.p0();
        let mut position = pos.get_mut(entity).unwrap();
        position.y = max;
    }
}
fn side(size: f32, meshes: &mut Assets<Mesh>) -> (Handle<Mesh>, Transform, Transform) {
    let mesh = meshes.add(Rectangle::new(2.0 * size, CARD_HEIGHT));
    let mut transform2 = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));
    transform2.translation.x = CARD_WIDTH / 2.0;
    let mut transform3 = Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0));
    transform3.translation.x = -CARD_WIDTH / 2.0;
    (mesh, transform2, transform3)
}
fn topbottom(size: f32, meshes: &mut Assets<Mesh>) -> (Handle<Mesh>, Transform, Transform) {
    let mesh2 = meshes.add(Rectangle::new(CARD_WIDTH, 2.0 * size));
    let mut transform4 = Transform::from_rotation(Quat::from_rotation_x(PI / 2.0));
    transform4.translation.y = -CARD_HEIGHT / 2.0;
    let mut transform5 = Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0));
    transform5.translation.y = CARD_HEIGHT / 2.0;
    (mesh2, transform4, transform5)
}
pub fn new_pile_at<'a>(
    pile: Pile,
    card_stock: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    card_back: Handle<StandardMaterial>,
    card_side: Handle<StandardMaterial>,
    transform: Transform,
    follow_mouse: bool,
    parent: Option<Entity>,
    id: Option<SyncObject>,
    sync_object: Option<SyncObjectMe>,
) -> Option<EntityCommands<'a>> {
    if pile.is_empty() {
        return None;
    }
    let card = pile.0.last().unwrap();
    let top = card.normal.image().clone();
    let material_handle = make_material(materials, top);
    let size = pile.len() as f32;
    let mut transform1 = Transform::from_rotation(Quat::from_rotation_y(PI));
    transform1.translation.z = -size;
    let (mesh, transform2, transform3) = side(size, meshes);
    let (mesh2, transform4, transform5) = topbottom(size, meshes);
    let mut ent = commands.spawn((
        pile,
        transform,
        Visibility::default(),
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SLEEP,
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::cuboid(CARD_WIDTH, CARD_HEIGHT, 2.0 * size),
        GravityScale(if follow_mouse || parent.is_some() {
            0.0
        } else {
            GRAVITY
        }),
        children![
            (
                Mesh3d(card_stock.clone()),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(0.0, 0.0, size),
            ),
            (Mesh3d(card_stock), MeshMaterial3d(card_back), transform1),
            (
                Mesh3d(mesh.clone()),
                MeshMaterial3d(card_side.clone()),
                transform2,
            ),
            (Mesh3d(mesh), MeshMaterial3d(card_side.clone()), transform3,),
            (
                Mesh3d(mesh2.clone()),
                MeshMaterial3d(card_side.clone()),
                transform4,
            ),
            (Mesh3d(mesh2), MeshMaterial3d(card_side), transform5)
        ],
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
    Some(ent)
}
pub fn is_reversed(transform: &Transform) -> bool {
    transform
        .rotation
        .to_euler(EulerRot::XYZ)
        .0
        .is_sign_positive()
}
pub fn repaint_face(
    mats: &mut Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    materials: &mut Assets<StandardMaterial>,
    card: &Card,
    children: &Children,
) {
    mats.get_mut(*children.first().unwrap()).unwrap().0 =
        make_material(materials, card.normal.image().clone());
}
pub fn adjust_meshes(
    pile: &Pile,
    children: &Children,
    meshes: &mut Assets<Mesh>,
    query: &mut Query<(&mut Mesh3d, &mut Transform), Without<Children>>,
    transform: &mut Transform,
    collider: &mut Collider,
) {
    let size = pile.len() as f32;
    *collider = Collider::cuboid(CARD_WIDTH, CARD_HEIGHT, 2.0 * size);
    let mut children = children.iter();
    let (_, mut top) = query.get_mut(children.next().unwrap()).unwrap();
    let delta = top.translation.z - size;
    top.translation.z -= delta;
    let (_, mut bottom) = query.get_mut(children.next().unwrap()).unwrap();
    bottom.translation.z += delta;
    transform.translation.y -= delta / 2.0;
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
}
