#![allow(clippy::shadow_reuse)]
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::mesh::Mesh;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Res, ResMut, Resource};
use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::Polyline;
#[derive(SystemParam)]
pub struct Asset<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub polylines: ResMut<'w, Assets<Polyline>>,
    pub polyline_materials: ResMut<'w, Assets<PolylineMaterial>>,
    pub text_mesh: Res<'w, TextMesh>,
    pub card: Res<'w, CardBase>,
}
#[derive(Resource)]
pub struct TextMesh {
    pub mesh: Handle<StandardMaterial>,
}
#[derive(Resource)]
pub struct CardBase {
    pub stock: Handle<Mesh>,
    pub back: Handle<StandardMaterial>,
    pub color: Handle<StandardMaterial>,
}
