use crate::assets::AssetManager;
use crate::events::repaint::GlobalIdMap;
use crate::focus::Hover;
use crate::pile::{ImageCard, PendingCards, Pile};
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::{Component, Event, ImageNode, KeyCode, Node, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::{Or, With};
use bevy_ecs::system::{Commands, Query, Res, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Event)]
pub struct ActivateAltMenu {
    pub entity: Entity,
}
#[derive(Event)]
pub struct RemoveAltMenu;
#[derive(Component)]
pub struct AltMenu {
    pub entity: Entity,
}
pub fn update_alt_menu(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Option<Single<&AltMenu>>,
    mut commands: Commands,
    has_image: Query<Option<&AltMenu>, Or<(With<ImageCard>, With<Pile>)>>,
    spatial: Spatial,
    hover: Hover,
) {
    if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        let Some(hit) = hover
            .get()
            .or_else(|| spatial.ray().map(|(r, _, _)| r.entity))
        else {
            if menu.is_some() {
                commands.trigger(RemoveAltMenu);
            }
            return;
        };
        let Ok(image) = has_image.get(hit) else {
            if menu.is_some() {
                commands.trigger(RemoveAltMenu);
            }
            return;
        };
        if image.is_some() {
            return;
        }
        if let Some(m) = menu {
            if m.entity != hit {
                commands.trigger(RemoveAltMenu);
                commands.trigger(ActivateAltMenu { entity: hit });
            }
        } else {
            commands.trigger(ActivateAltMenu { entity: hit });
        }
    } else if keys.any_just_released([KeyCode::AltLeft, KeyCode::AltRight]) && menu.is_some() {
        commands.trigger(RemoveAltMenu);
    }
}
#[query_fn]
pub fn on_activate_alt_menu(
    event: On<ActivateAltMenu>,
    piles: Query<(&Pile, &Transform)>,
    images: Query<&ImageCard>,
    mut commands: Commands,
    assets: AssetManager,
    global_map: Res<GlobalIdMap>,
) {
    let card = match (piles.get(event.entity), images.get(event.entity)) {
        (Ok(pile), Err(_)) => pile.pile.get_card(pile.transform.rotation),
        (Err(_), Ok(image)) => {
            let ent = global_map.map.get(&image.global_id).unwrap();
            let pile = piles.get(*ent).unwrap();
            pile.pile
                .iter()
                .find(|c| c.global_id == image.global_id)
                .unwrap()
        }
        _ => unreachable!(),
    };
    let bundle = (
        AltMenu {
            entity: event.entity,
        },
        Node { ..Node::default() },
        ImageCard {
            id: card.data.id,
            quality: card.quality,
            flipped: card.flipped,
            global_id: card.global_id,
        },
    );
    if let Some(handles) = card.face_handles() {
        commands.spawn((bundle, ImageNode::new(handles.image())));
    } else {
        commands.spawn((
            bundle,
            PendingCards,
            ImageNode::new(assets.card.back_image.clone()),
        ));
    }
}
pub fn on_remove_alt_menu(
    _: On<RemoveAltMenu>,
    ent: Single<Entity, With<AltMenu>>,
    mut commands: Commands,
) {
    commands.entity(*ent).despawn();
}
