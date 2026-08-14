#![allow(clippy::shadow_reuse)]
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::material::AlphaMode;
use bevy::mesh::Mesh;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Res, ResMut, Resource};
use importer::card::Handles;
#[derive(SystemParam)]
pub struct AssetManager<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub card: Res<'w, CardBase>,
    pub text_mesh: Res<'w, TextMesh>,
}
#[derive(Resource)]
pub struct CardBase {
    pub stock: Handle<Mesh>,
    pub back: Handle<StandardMaterial>,
    pub back_image: Handle<Image>,
    pub color: Handle<StandardMaterial>,
}
#[derive(Resource)]
pub struct TextMesh {
    pub mesh: Handle<StandardMaterial>,
}
impl AssetManager<'_> {
    pub fn text(&mut self) -> Handle<StandardMaterial> {
        self.text_mesh.mesh.clone()
    }
    pub fn register_card(&mut self, image: Handle<Image>) -> Handles {
        let material = self.materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            alpha_mode: AlphaMode::AlphaToCoverage,
            unlit: true,
            ..StandardMaterial::default()
        });
        Handles { image, material }
    }
}
