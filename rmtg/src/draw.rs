use crate::CARD_WIDTH;
use crate::drag::Dragging;
use crate::events::hover::{AddHover, HoveredObject, RemoveHover};
use crate::keybinds::Keybind;
use crate::pile::Pile;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::Transform;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res};
use bevy_query_fn_macro::query_fn;
#[query_fn]
pub fn update_draw(
    keybinds: Res<ButtonInput<Keybind>>,
    hovered: Query<(&mut Pile, &Transform, Entity), With<HoveredObject>>,
    spatial: Spatial,
    mut commands: Commands,
) {
    if !keybinds.just_pressed(Keybind::Draw) {
        return;
    }
    let Some((_, mut pos, _)) = spatial.ray() else {
        return;
    };
    pos.y += CARD_WIDTH;
    let mut cards = Vec::new();
    for mut pile in hovered {
        if pile.pile.len() == 1 {
            commands.entity(pile.entity).despawn();
        } else {
            commands.trigger(RemoveHover {
                entity: pile.entity,
            });
        }
        let card = pile.pile.take_card(pile.transform.rotation);
        cards.push(card);
    }
    if cards.is_empty() {
        return;
    }
    let ent = commands
        .spawn((
            Pile::new(cards).bundle(),
            Transform::from_translation(pos),
            Dragging { pos },
        ))
        .id();
    commands.trigger(AddHover::new(ent, HoveredObject { held: false }));
}
