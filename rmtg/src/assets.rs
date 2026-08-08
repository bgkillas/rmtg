#![allow(clippy::shadow_reuse)]
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::material::AlphaMode;
use bevy::mesh::Mesh;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Res, ResMut, Resource};
use bevy_rich_text3d::TextAtlas;
use importer::card::{Handles, SubCard};
#[derive(SystemParam)]
pub struct Asset<'w> {
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub images: ResMut<'w, Assets<Image>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub card: Res<'w, CardBase>,
}
#[derive(Resource)]
pub struct CardBase {
    pub stock: Handle<Mesh>,
    pub back: Handle<StandardMaterial>,
    pub color: Handle<StandardMaterial>,
}
impl Asset<'_> {
    pub fn text(&mut self) -> Handle<StandardMaterial> {
        self.materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..StandardMaterial::default()
        })
    }
    pub fn register(&mut self, card: &mut SubCard, front: Image, back: Option<Image>) {
        if let Some(back_image) = back {
            let image = self.images.add(back_image);
            let material = self.materials.add(StandardMaterial {
                base_color_texture: Some(image.clone()),
                alpha_mode: AlphaMode::Premultiplied,
                unlit: true,
                ..StandardMaterial::default()
            });
            card.data.back.as_mut().unwrap().handles = Some(Handles { image, material });
        }
        let image = self.images.add(front);
        let material = self.materials.add(StandardMaterial {
            base_color_texture: Some(image.clone()),
            alpha_mode: AlphaMode::Premultiplied,
            unlit: true,
            ..StandardMaterial::default()
        });
        card.data.front.handles = Some(Handles { image, material });
    }
}
