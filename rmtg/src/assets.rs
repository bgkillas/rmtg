#![allow(clippy::shadow_reuse)]
use bevy::asset::{Assets, Handle};
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::material::AlphaMode;
use bevy::mesh::Mesh;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Res, ResMut, Resource};
use bevy_rich_text3d::TextAtlas;
use importer::card::{Handles, MaybeHandles, SubCard};
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
    pub back_image: Handle<Image>,
    pub color: Handle<StandardMaterial>,
}
impl Asset<'_> {
    pub fn text(&mut self) -> Handle<StandardMaterial> {
        self.materials.add(StandardMaterial {
            base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
            alpha_mode: AlphaMode::AlphaToCoverage,
            unlit: true,
            ..StandardMaterial::default()
        })
    }
    pub fn register(&mut self, card: &mut SubCard, front: Option<Image>, back: Option<Image>) {
        let front_handle = front.map(|i| self.images.add(i));
        let back_handle = back.map(|i| self.images.add(i));
        self.register_handles(card, front_handle, back_handle);
    }
    pub fn register_handles(
        &mut self,
        card: &mut SubCard,
        front: Option<Handle<Image>>,
        back: Option<Handle<Image>>,
    ) {
        if let Some(back_data) = &mut card.data.back {
            if let Some(image) = back {
                let material = self.materials.add(StandardMaterial {
                    base_color_texture: Some(image.clone()),
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    unlit: true,
                    ..StandardMaterial::default()
                });
                back_data.handles = MaybeHandles::Some(Handles { image, material });
            } else {
                back_data.is_oracle = true;
            }
        }
        if let Some(image) = front {
            let material = self.materials.add(StandardMaterial {
                base_color_texture: Some(image.clone()),
                alpha_mode: AlphaMode::AlphaToCoverage,
                unlit: true,
                ..StandardMaterial::default()
            });
            card.data.front.handles = MaybeHandles::Some(Handles { image, material });
        } else {
            card.data.front.is_oracle = true;
        }
    }
}
