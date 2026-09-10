use crate::assets::AssetManager;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::{MouseButton, Transform};
use bevy_ecs::children;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::Bundle;
use bevy_ecs::system::Res;
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct PingDrag;
pub fn drag_object(assets: &AssetManager, from: Vec3, to: Vec3) -> impl Bundle {
    (
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.outlines.players[0].clone()),
        Transform::from_translation((from + to) / 2.0),
        children![
            (
                Mesh3d(assets.meshes.sphere.clone()),
                MeshMaterial3d(assets.outlines.players[0].clone()),
                Transform::from_translation(from)
            ),
            (
                Mesh3d(assets.meshes.sphere.clone()),
                MeshMaterial3d(assets.outlines.players[0].clone()),
                Transform::from_translation(to)
            )
        ],
    )
}
#[query_fn]
pub fn update_ping_drag(button: Res<ButtonInput<MouseButton>>) {
    if button.just_pressed(MouseButton::Middle) {
        //TODO
    } else if button.just_released(MouseButton::Middle) {
        //TODO
    } else {
        //TODO
    }
}
