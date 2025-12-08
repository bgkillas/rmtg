use crate::setup::{MAT_WIDTH, Wall};
use crate::sync::{CameraInd, CursorInd, SyncObjectMe};
use crate::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use bevy_tangled::PeerId;
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
    let size = pile.len() as f32 * CARD_THICKNESS;
    let transform = Transform::from_xyz(v.x, size / 2.0, v.y);
    println!("{size:?}");
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
    ents: &mut Query<(&Collider, &mut Transform), Without<Wall>>,
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
        translation.y = max;
        position.y = max;
    }
    let (_, mut t) = ents.get_mut(entity).unwrap();
    t.translation.y = translation.y
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
    let card = pile.last();
    let top = card.normal.image().clone();
    let material_handle = make_material(materials, top);
    let size = pile.len() as f32 * CARD_THICKNESS;
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
        Collider::cuboid(CARD_WIDTH, size, CARD_HEIGHT),
        CollisionEventsEnabled,
        GravityScale(if follow_mouse || parent.is_some() {
            0.0
        } else {
            GRAVITY
        }),
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
    (transform.rotation * Vec3::Y).y < 0.0
}
pub fn repaint_face(
    mats: &mut Query<&mut MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    card: &SubCard,
    children: &Children,
) {
    mats.get_mut(*children.first().unwrap()).unwrap().0 =
        make_material(materials, card.normal.image().clone());
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
        ),
    >,
    transform: &mut Transform,
    collider: &mut Collider,
) {
    let size = pile.len() as f32 * CARD_THICKNESS;
    *collider = Collider::cuboid(CARD_WIDTH, size, CARD_HEIGHT);
    let mut children = children.iter();
    let (_, mut top) = query.get_mut(children.next().unwrap()).unwrap();
    let delta = top.translation.y - size / 2.0;
    top.translation.y -= delta;
    let (_, mut bottom) = query.get_mut(children.next().unwrap()).unwrap();
    bottom.translation.y += delta;
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
