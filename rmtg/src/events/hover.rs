use crate::assets::Asset;
use crate::keybinds::{Keybind, Keybinds};
use crate::shapes::{OUTLINE_COLOR, OUTLINE_DEPTH_BIAS};
use crate::spatial::Spatial;
use crate::{CARD_THICKNESS, PLAYER};
use bevy::math::bounding::Aabb2d;
use bevy::math::{Isometry2d, Vec2, Vec3};
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
#[derive(Event)]
pub struct SpawnBoxSelect {
    pub pos: Vec3,
}
const BOX_SELECT_RADIUS: f32 = 4.0 * CARD_THICKNESS;
pub fn spawn_box_select(mut event: On<SpawnBoxSelect>, mut commands: Commands) {
    let vec = Vec2::new(event.pos.x, event.pos.z);
    event.pos.y += BOX_SELECT_RADIUS;
    commands.spawn((
        BoxSelect { start: vec },
        Transform::from_translation(event.pos),
    ));
}
pub fn update_box_select(
    box_select: Option<Single<(Entity, &mut BoxSelect)>>,
    olds: Query<(), With<Hovered>>,
    hoverable: Query<(Entity, &Transform), With<Hoverable>>,
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
    let Some((_, pos)) = spatial.ray() else {
        return;
    };
    let vec = Vec2::new(pos.x, pos.z);
    let mut aabb = Aabb2d::from_point_cloud(Isometry2d::default(), &[select.start, vec]);
    aabb.min -= Vec2::splat(BOX_SELECT_RADIUS);
    aabb.max += Vec2::splat(BOX_SELECT_RADIUS);
    for (ent, trans) in hoverable {
        let splat = Vec2::new(trans.translation.x, trans.translation.z);
        if aabb.closest_point(splat) != splat {
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
    let Some((hit, pos)) = spatial.ray() else {
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
