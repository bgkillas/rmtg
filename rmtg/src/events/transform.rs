use crate::events::hover::HoveredObject;
use crate::events::repaint::Repaint;
use crate::keybinds::Keybind;
use crate::pile::{FlippedState, ImageCard, Pile};
use crate::ui::alt_menu::AltMenu;
use bevy::input::ButtonInput;
use bevy::prelude::{ImageNode, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::Single;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_query_fn_macro::query_fn;
use std::mem;
#[derive(Event)]
pub struct Transformed {
    pub entity: Entity,
    pub state: bool,
}
#[query_fn]
pub fn trigger_transform(
    hovered: Query<(Entity, &mut Pile, &Transform), With<HoveredObject>>,
    keybinds: Res<ButtonInput<Keybind>>,
    altmenu: Option<Single<(&mut ImageNode, &mut ImageCard), With<AltMenu>>>,
    mut commands: Commands,
) {
    if keybinds.just_pressed(Keybind::Transform) {
        if let Some(mut image) = altmenu {
            if let Some(mut back) = mem::take(&mut image.image_card.back_handle) {
                mem::swap(&mut image.image_node.image, &mut back);
                image.image_card.back_handle = Some(back);
            }
            return;
        }
        for mut obj in hovered {
            if FlippedState::from(obj.transform.rotation).flipped() {
                continue;
            }
            let card = obj.pile.first_mut();
            if card.data.transformable {
                card.transformed = !card.transformed;
                commands.trigger(Transformed {
                    entity: obj.entity,
                    state: card.transformed,
                });
                commands.trigger(Repaint::new(obj.entity));
            }
        }
    }
}
