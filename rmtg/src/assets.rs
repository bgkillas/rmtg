#![allow(clippy::shadow_reuse)]
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::mesh::Mesh;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Res, ResMut, Resource};
use bevy_polyline::material::PolylineMaterial;
use bevy_polyline::polyline::Polyline;
use importer::card::{Handles, SubCard};
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
impl Asset<'_> {
    pub fn register(&mut self, card: &mut SubCard, front: Image, back: Option<Image>) {
        if let Some(back_image) = back {
            let image = self.images.add(back_image);
            let material = self.materials.add(StandardMaterial {
                base_color_texture: Some(image.clone()),
                unlit: true,
                ..StandardMaterial::default()
            });
            card.data.back.as_mut().unwrap().handles = Some(Handles { image, material });
        }
        let image = self.images.add(front);
        let material = self.materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            unlit: true,
            ..StandardMaterial::default()
        });
        card.data.front.handles = Some(Handles { image, material });
    }
}
