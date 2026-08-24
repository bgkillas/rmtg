use crate::assets::AssetManager;
use crate::focus::Hover;
use crate::keybinds::Keybind;
use crate::pile::Pile;
use crate::shapes::ShapeOutline as _;
use crate::shapes::drag_outline::DragOutline;
use crate::spatial::Spatial;
use crate::startup::wall_aabb;
use avian3d::prelude::{Collider, ColliderAabb};
use avian3d::spatial_query::SpatialQueryFilter;
use bevy::asset::Assets;
use bevy::input::ButtonInput;
use bevy::math::bounding::{Aabb2d, Aabb3d, BoundingVolume as _, IntersectsVolume as _};
use bevy::math::{Isometry2d, Quat, Vec2, Vec3, Vec3A, Vec3Swizzles as _};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Children, Commands, Component, Entity, EntityEvent, On, Query, Transform, With,
};
use bevy_ecs::event::Event;
use bevy_ecs::system::{Res, ResMut, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Component, Clone)]
pub struct Hoverable;
#[derive(Component, Clone, Copy)]
pub struct HoveredObject {
    pub held: bool,
}
#[derive(EntityEvent)]
pub struct AddHover {
    pub entity: Entity,
    pub hovered: HoveredObject,
}
#[derive(EntityEvent)]
pub struct RemoveHover {
    pub entity: Entity,
}
impl AddHover {
    #[must_use]
    pub fn new(entity: Entity, hovered: HoveredObject) -> Self {
        Self { entity, hovered }
    }
}
impl RemoveHover {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn add_hover(
    event: On<AddHover>,
    mut commands: Commands,
    children: Query<&Children>,
    is_pile: Query<(), With<Pile>>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    asset: AssetManager,
) {
    let childs = children.get(event.entity).unwrap();
    if is_pile.contains(event.entity) {
        for &child in &childs[3..6] {
            let mut mat = query.get_mut(child).unwrap();
            mat.0 = asset.outlines.players[0].clone();
        }
    } else {
        let mut mat = query.get_mut(childs[0]).unwrap();
        mat.0 = asset.outlines.players[0].clone();
    }
    commands.entity(event.entity).insert(event.hovered);
}
pub fn remove_hover(
    event: On<RemoveHover>,
    mut commands: Commands,
    children: Query<&Children>,
    is_pile: Query<(), With<Pile>>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    asset: AssetManager,
) {
    let childs = children.get(event.entity).unwrap();
    if is_pile.contains(event.entity) {
        for &child in &childs[3..6] {
            let mut mat = query.get_mut(child).unwrap();
            mat.0 = asset.outlines.default.clone();
        }
    } else {
        let mut mat = query.get_mut(childs[0]).unwrap();
        mat.0 = asset.outlines.default.clone();
    }
    commands.entity(event.entity).remove::<HoveredObject>();
}
#[derive(Component)]
pub struct BoxSelect {
    pub start: Vec2,
}
#[derive(EntityEvent)]
pub struct UpdateBoxSelect {
    pub entity: Entity,
    pub vec: Vec2,
}
#[derive(Event)]
pub struct SpawnBoxSelect {
    pub pos: Vec3,
}
pub fn spawn_box_select(
    mut event: On<SpawnBoxSelect>,
    mut commands: Commands,
    asset: AssetManager,
) {
    let vec = event.pos.xz();
    event.pos.y += DragOutline::THICKNESS;
    let entity = commands
        .spawn((
            BoxSelect { start: vec },
            Transform::from_translation(event.pos),
            MeshMaterial3d(asset.outlines.players[0].clone()),
        ))
        .id();
    commands.trigger(UpdateBoxSelect { entity, vec });
}
#[query_fn]
pub fn update_box_select_mesh(
    event: On<UpdateBoxSelect>,
    mut box_selects: Query<(&mut Transform, &BoxSelect)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut box_select = box_selects.get_mut(event.entity).unwrap();
    let vec = (box_select.box_select.start + event.vec) / 2.0;
    box_select.transform.translation.x = vec.x;
    box_select.transform.translation.z = vec.y;
    let drag = DragOutline {
        x: (event.vec.x - box_select.box_select.start.x).abs() / 2.0,
        y: (event.vec.y - box_select.box_select.start.y).abs() / 2.0,
    };
    let mesh = drag.mesh();
    commands
        .entity(event.entity)
        .insert(Mesh3d(meshes.add(mesh)));
}
#[query_fn]
pub fn update_box_select(
    box_select: Single<(Entity, &mut BoxSelect)>,
    olds: Query<(), With<HoveredObject>>,
    hoverable: Query<(Entity, &ColliderAabb), With<Hoverable>>,
    spatial: Spatial,
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
) {
    if !keybinds.pressed(Keybind::Select) && !keybinds.pressed(Keybind::HoldSelect) {
        commands.entity(box_select.entity).despawn();
        return;
    }
    let Some((_, _, pos)) = spatial.ray() else {
        return;
    };
    let vec = pos.xz();
    commands.trigger(UpdateBoxSelect {
        entity: box_select.entity,
        vec,
    });
    let mut aabb =
        Aabb2d::from_point_cloud(Isometry2d::default(), &[box_select.box_select.start, vec]);
    aabb.min -= Vec2::splat(DragOutline::THICKNESS);
    aabb.max += Vec2::splat(DragOutline::THICKNESS);
    for hover_ent in hoverable {
        let splat = Aabb2d {
            min: hover_ent.collider_aabb.min.xz(),
            max: hover_ent.collider_aabb.max.xz(),
        };
        if !aabb.intersects(&splat)
            && olds.contains(hover_ent.entity)
            && !keybinds.pressed(Keybind::HoldSelect)
        {
            commands.trigger(RemoveHover::new(hover_ent.entity));
        }
    }
    let caabb = Aabb3d {
        min: Vec3A::new(aabb.min.x, wall_aabb().min.y, aabb.min.y),
        max: Vec3A::new(aabb.max.x, wall_aabb().max.y, aabb.max.y),
    };
    spatial.spatial.shape_intersections_callback(
        &Collider::cuboid(
            caabb.max.x - caabb.min.x,
            caabb.max.y - caabb.min.y,
            caabb.max.z - caabb.min.z,
        ),
        caabb.center().to_vec3(),
        Quat::default(),
        &SpatialQueryFilter::DEFAULT,
        |ent| {
            if hoverable.contains(ent) && !olds.contains(ent) {
                commands.trigger(AddHover::new(ent, HoveredObject { held: true }));
            }
            true
        },
    );
}
#[query_fn]
pub fn update_hover(
    box_select: Option<Single<(), With<BoxSelect>>>,
    olds: Query<(Entity, &HoveredObject)>,
    hoverable: Query<(), With<Hoverable>>,
    keybinds: Res<ButtonInput<Keybind>>,
    spatial: Spatial,
    mut commands: Commands,
    hover: Hover,
) {
    let Some((hit, _, pos)) = spatial.ray() else {
        return;
    };
    if !hoverable.contains(hit.entity) {
        for old in olds {
            if (!old.hovered_object.held && !keybinds.pressed(Keybind::Select))
                || keybinds.just_pressed(Keybind::Select)
            {
                commands.trigger(RemoveHover::new(old.entity));
            }
        }
        if box_select.is_none()
            && (keybinds.just_pressed(Keybind::HoldSelect)
                || keybinds.just_pressed(Keybind::Select))
        {
            commands.trigger(SpawnBoxSelect { pos });
        }
        return;
    }
    if hover.get().is_some() {
        return;
    }
    if keybinds.just_pressed(Keybind::HoldSelect) {
        if olds
            .iter()
            .any(|old| old.entity == hit.entity && old.hovered_object.held)
        {
            commands.trigger(RemoveHover::new(hit.entity));
        } else {
            commands.trigger(AddHover::new(hit.entity, HoveredObject { held: true }));
        }
    } else if keybinds.pressed(Keybind::Select) {
        if keybinds.just_pressed(Keybind::Select)
            && olds
                .iter()
                .all(|old| old.entity != hit.entity || !old.hovered_object.held)
        {
            for old in olds {
                if old.entity != hit.entity {
                    commands.trigger(RemoveHover::new(old.entity));
                }
            }
        }
    } else if olds.iter().all(|old| old.entity != hit.entity) {
        for old in olds {
            if !old.hovered_object.held {
                commands.trigger(RemoveHover::new(old.entity));
            }
        }
        commands.trigger(AddHover::new(hit.entity, HoveredObject { held: false }));
    }
}
