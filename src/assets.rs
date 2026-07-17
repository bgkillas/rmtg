#![allow(clippy::shadow_reuse)]
use bevy::asset::Assets;
use bevy::ecs::system::SystemParam;
use bevy::mesh::Mesh;
use bevy::pbr::StandardMaterial;
use bevy::prelude::ResMut;
use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::Polyline;
#[derive(SystemParam)]
pub struct Asset<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub polylines: ResMut<'w, Assets<Polyline>>,
    pub polyline_materials: ResMut<'w, Assets<PolylineMaterial>>,
}
