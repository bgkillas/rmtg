use crate::assets::AssetManager;
use crate::net::Endpoint;
use crate::{CARD_THICKNESS, CARD_WIDTH};
use avian3d::parry::glamx::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::Transform;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::In;
use bevy_ecs::query::{With, Without};
use bevy_ecs::system::{Commands, Query};
use bevy_p2p::iroh::EndpointId;
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct CameraIndicator;
#[derive(Component)]
pub struct CursorIndicator;
pub const CURSOR_SCALE: f32 = 4.0 * CARD_THICKNESS;
impl CameraIndicator {
    pub fn bundle(assets: &AssetManager, peer: EndpointId, pos: Vec3) -> impl Bundle {
        (
            Self,
            Endpoint::from(peer),
            Transform::from_translation(pos).with_scale(Vec3::splat(CARD_WIDTH / 2.0)),
            Mesh3d(assets.meshes.sphere.clone()),
            MeshMaterial3d(assets.outlines.players[0].clone()),
        )
    }
}
impl CursorIndicator {
    pub fn bundle(assets: &AssetManager, peer: EndpointId, pos: Vec3) -> impl Bundle {
        (
            Self,
            Endpoint::from(peer),
            Transform::from_translation(pos).with_scale(Vec3::splat(CURSOR_SCALE)),
            Mesh3d(assets.meshes.sphere.clone()),
            MeshMaterial3d(assets.outlines.players[0].clone()),
        )
    }
}
#[query_fn]
pub fn move_camera(
    In((peer, camera_pos, cursor_pos)): In<(EndpointId, Vec3, Vec3)>,
    mut cameras: Query<
        (&Endpoint, &mut Transform),
        (With<CameraIndicator>, Without<CursorIndicator>),
    >,
    mut cursors: Query<
        (&Endpoint, &mut Transform),
        (With<CursorIndicator>, Without<CameraIndicator>),
    >,
    mut commands: Commands,
    assets: AssetManager,
) {
    if let Some(mut camera) = cameras.iter_mut().find(|c| c.endpoint.peer == peer) {
        camera.transform.translation = camera_pos;
    } else {
        commands.spawn(CameraIndicator::bundle(&assets, peer, camera_pos));
    }
    if let Some(mut cursor) = cursors.iter_mut().find(|c| c.endpoint.peer == peer) {
        cursor.transform.translation = cursor_pos;
    } else {
        commands.spawn(CursorIndicator::bundle(&assets, peer, cursor_pos));
    }
}
