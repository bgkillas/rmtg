use crate::CARD_THICKNESS;
use crate::assets::AssetManager;
use crate::camera_indicator::CURSOR_SCALE;
use crate::keybinds::Keybind;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::math::{Quat, Vec3};
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::Transform;
use bevy_ecs::children;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Bundle;
use bevy_ecs::system::{Commands, Query, Res, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct PingDrag {
    pub from: Vec3,
}
#[derive(Component)]
pub struct DragObject;
impl DragObject {
    pub fn bundle(assets: &AssetManager, from: Vec3, to: Vec3) -> impl Bundle {
        let orig = (from + to) / 2.0;
        let length = (to - from).length().max(CARD_THICKNESS / 64.0);
        let transform = Transform::from_translation(orig)
            .with_scale(Vec3::new(CURSOR_SCALE, length, CURSOR_SCALE))
            .with_rotation(Quat::from_rotation_arc_colinear(
                Vec3::Y,
                (to - orig).normalize(),
            ));
        (
            Self,
            Mesh3d(assets.meshes.cylinder.clone()),
            MeshMaterial3d(assets.outlines.players[0].clone()),
            transform,
            children![
                (
                    Mesh3d(assets.meshes.sphere.clone()),
                    MeshMaterial3d(assets.outlines.players[0].clone()),
                    Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)).with_scale(Vec3::new(
                        1.0,
                        CURSOR_SCALE * transform.scale.y.recip(),
                        1.0
                    ))
                ),
                (
                    Mesh3d(assets.meshes.sphere.clone()),
                    MeshMaterial3d(assets.outlines.players[0].clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)).with_scale(Vec3::new(
                        1.0,
                        CURSOR_SCALE * transform.scale.y.recip(),
                        1.0
                    ))
                )
            ],
        )
    }
}
#[derive(Event)]
pub struct MoveDragObject {
    pub entity: Entity,
    pub from: Vec3,
    pub to: Vec3,
}
impl MoveDragObject {
    pub fn new(entity: Entity, from: Vec3, to: Vec3) -> Self {
        Self { entity, from, to }
    }
}
#[query_fn]
pub fn move_drag_object(
    event: On<MoveDragObject>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform>,
) {
    let childs = children.get(event.entity).unwrap();
    let [mut t0, mut t1, mut t2] = transforms
        .get_many_mut([event.entity, childs[0], childs[1]])
        .unwrap();
    let orig = (event.from + event.to) / 2.0;
    t0.translation = orig;
    t0.scale.y = (event.to - event.from).length().max(CARD_THICKNESS / 64.0);
    t0.rotation = Quat::from_rotation_arc_colinear(Vec3::Y, (event.to - orig).normalize());
    t1.scale.y = CURSOR_SCALE * t0.scale.y.recip();
    t2.scale.y = CURSOR_SCALE * t0.scale.y.recip();
}
#[query_fn]
pub fn update_ping_drag(
    keybinds: Res<ButtonInput<Keybind>>,
    mut commands: Commands,
    assets: AssetManager,
    spatial: Spatial,
    ping_drag: Option<Single<(Entity, &PingDrag)>>,
) {
    if keybinds.just_pressed(Keybind::Ping) {
        let Some((_, from, _)) = spatial.ray() else {
            return;
        };
        commands.spawn((PingDrag { from }, DragObject::bundle(&assets, from, from)));
    } else if keybinds.just_released(Keybind::Ping) {
        if let Some(ping) = ping_drag {
            commands.entity(ping.entity).despawn();
        }
    } else if let Some(ping) = ping_drag {
        let Some((_, hit, _)) = spatial.ray() else {
            return;
        };
        commands.trigger(MoveDragObject::new(ping.entity, ping.ping_drag.from, hit));
    }
}
