use crate::PLAYER;
use crate::assets::Asset;
use crate::keybinds::{Keybind, Keybinds};
use crate::shapes::drag::DragOutline;
use crate::shapes::{OUTLINE_COLOR, OUTLINE_DEPTH_BIAS, ShapeOutline as _};
use crate::spatial::Spatial;
use avian3d::prelude::ColliderAabb;
use bevy::math::bounding::{Aabb2d, IntersectsVolume as _};
use bevy::math::{Isometry2d, Vec2, Vec3, Vec3Swizzles as _};
use bevy::mesh::Mesh3d;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Children, Commands, Component, Entity, EntityEvent, On, Query, Transform, With,
};
use bevy_ecs::event::Event;
use bevy_ecs::system::Single;
#[derive(Component, Clone)]
pub struct Hoverable;
#[derive(Component, Clone, Copy)]
pub struct Hovered {
    pub held: bool,
}
#[derive(EntityEvent)]
pub struct AddHover {
    pub entity: Entity,
    pub hovered: Hovered,
}
#[derive(EntityEvent)]
pub struct RemoveHover {
    pub entity: Entity,
}
impl AddHover {
    #[must_use]
    pub fn new(entity: Entity, hovered: Hovered) -> Self {
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
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut asset: Asset,
) {
    let childs = children.get(event.entity).unwrap();
    let mut mat = query.get_mut(childs[0]).unwrap();
    mat.0 = asset.materials.add(StandardMaterial {
        base_color: PLAYER[0],
        unlit: true,
        depth_bias: OUTLINE_DEPTH_BIAS,
        ..StandardMaterial::default()
    });
    commands.entity(event.entity).insert(event.hovered);
}
pub fn remove_hover(
    event: On<RemoveHover>,
    mut commands: Commands,
    children: Query<&Children>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut asset: Asset,
) {
    let childs = children.get(event.entity).unwrap();
    let mut mat = query.get_mut(childs[0]).unwrap();
    mat.0 = asset.materials.add(StandardMaterial {
        base_color: OUTLINE_COLOR,
        unlit: true,
        depth_bias: OUTLINE_DEPTH_BIAS,
        ..StandardMaterial::default()
    });
    commands.entity(event.entity).remove::<Hovered>();
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
pub fn spawn_box_select(mut event: On<SpawnBoxSelect>, mut commands: Commands, mut asset: Asset) {
    let vec = event.pos.xz();
    event.pos.y += DragOutline::THICKNESS;
    let entity = commands
        .spawn((
            BoxSelect { start: vec },
            Transform::from_translation(event.pos),
            MeshMaterial3d(asset.materials.add(StandardMaterial {
                unlit: true,
                base_color: PLAYER[0],
                ..StandardMaterial::default()
            })),
        ))
        .id();
    commands.trigger(UpdateBoxSelect { entity, vec });
}
pub fn update_box_select_mesh(
    event: On<UpdateBoxSelect>,
    mut box_select: Query<(&mut Transform, &BoxSelect)>,
    mut commands: Commands,
    mut asset: Asset,
) {
    let (mut transform, select) = box_select.get_mut(event.entity).unwrap();
    let vec = (select.start + event.vec) / 2.0;
    transform.translation.x = vec.x;
    transform.translation.z = vec.y;
    let drag = DragOutline {
        x: (event.vec.x - select.start.x).abs() / 2.0,
        y: (event.vec.y - select.start.y).abs() / 2.0,
    };
    let mesh = drag.mesh();
    commands
        .entity(event.entity)
        .insert(Mesh3d(asset.meshes.add(mesh)));
}
pub fn update_box_select(
    box_select: Option<Single<(Entity, &mut BoxSelect)>>,
    olds: Query<(), With<Hovered>>,
    hoverable: Query<(Entity, &ColliderAabb), With<Hoverable>>,
    spatial: Spatial,
    mut commands: Commands,
    keybinds: Keybinds,
) {
    let Some((entity, select)) = box_select.map(Single::into_inner) else {
        return;
    };
    if !keybinds.pressed(Keybind::Select) && !keybinds.pressed(Keybind::HoldSelect) {
        commands.entity(entity).despawn();
        return;
    }
    let Some((_, _, pos)) = spatial.ray() else {
        return;
    };
    let vec = pos.xz();
    commands.trigger(UpdateBoxSelect { entity, vec });
    let mut aabb = Aabb2d::from_point_cloud(Isometry2d::default(), &[select.start, vec]);
    aabb.min -= Vec2::splat(DragOutline::THICKNESS);
    aabb.max += Vec2::splat(DragOutline::THICKNESS);
    for (ent, caabb) in hoverable {
        let splat = Aabb2d {
            min: caabb.min.xz(),
            max: caabb.max.xz(),
        };
        if !aabb.intersects(&splat) {
            if olds.contains(ent) {
                commands.trigger(RemoveHover::new(ent));
            }
            continue;
        }
        if olds.contains(ent) {
            continue;
        }
        commands.trigger(AddHover::new(ent, Hovered { held: true }));
    }
}
pub fn update_hover(
    box_select: Option<Single<(), With<BoxSelect>>>,
    olds: Query<(Entity, &Hovered)>,
    hoverable: Query<(), With<Hoverable>>,
    keybinds: Keybinds,
    spatial: Spatial,
    mut commands: Commands,
) {
    let Some((hit, _, pos)) = spatial.ray() else {
        return;
    };
    if !hoverable.contains(hit.entity) {
        for (ent, hovered) in olds {
            if (!hovered.held && !keybinds.pressed(Keybind::Select))
                || keybinds.just_pressed(Keybind::Select)
            {
                commands.trigger(RemoveHover::new(ent));
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
    if keybinds.just_pressed(Keybind::HoldSelect) {
        if olds.iter().any(|(e, h)| e == hit.entity && h.held) {
            commands.trigger(RemoveHover::new(hit.entity));
        } else {
            commands.trigger(AddHover::new(hit.entity, Hovered { held: true }));
        }
    } else if keybinds.pressed(Keybind::Select) {
        if keybinds.just_pressed(Keybind::Select)
            && olds.iter().all(|(e, h)| e != hit.entity || !h.held)
        {
            for (ent, _) in olds {
                if ent != hit.entity {
                    commands.trigger(RemoveHover::new(ent));
                }
            }
        }
    } else if olds.iter().all(|(e, _)| e != hit.entity) {
        for (ent, hovered) in olds {
            if !hovered.held {
                commands.trigger(RemoveHover::new(ent));
            }
        }
        commands.trigger(AddHover::new(hit.entity, Hovered { held: false }));
    }
}
