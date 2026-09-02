use crate::assets::AssetManager;
use crate::pile::{ImageCard, PendingCards, Pile};
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::{Component, Event, ImageNode, KeyCode, Node, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::With;
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
    spatial: Spatial,
) {
    if keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        let Some((hit, _, _)) = spatial.ray() else {
            return;
        };
        if let Some(m) = menu {
            if m.entity != hit.entity {
                commands.trigger(RemoveAltMenu);
                commands.trigger(ActivateAltMenu { entity: hit.entity });
            }
        } else {
            commands.trigger(ActivateAltMenu { entity: hit.entity });
        }
    } else if keys.any_just_released([KeyCode::AltLeft, KeyCode::AltRight]) && menu.is_some() {
        commands.trigger(RemoveAltMenu);
    }
}
#[query_fn]
pub fn on_activate_alt_menu(
    event: On<ActivateAltMenu>,
    piles: Query<(&Pile, &Transform)>,
    mut commands: Commands,
    assets: AssetManager,
) {
    let Ok(ent) = piles.get(event.entity) else {
        return;
    };
    let card = ent.pile.get_card(ent.transform.rotation);
    let bundle = (
        AltMenu {
            entity: event.entity,
        },
        Node { ..Node::default() },
        ImageCard {
            id: card.data.id,
            quality: card.quality,
            flipped: card.flipped,
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
