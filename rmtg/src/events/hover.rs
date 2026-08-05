use crate::PLAYER;
use crate::assets::Asset;
use crate::keybinds::{Keybind, Keybinds};
use crate::shapes::{OUTLINE_COLOR, OUTLINE_DEPTH_BIAS};
use crate::spatial::Spatial;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Children, Commands, Component, Entity, EntityEvent, On, Query, With};
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
pub fn update_hover(
    olds: Query<(Entity, &Hovered)>,
    hoverable: Query<(), With<Hoverable>>,
    keybinds: Keybinds,
    spatial: Spatial,
    mut commands: Commands,
) {
    let Some((hit, _)) = spatial.ray() else {
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
