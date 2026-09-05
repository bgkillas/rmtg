use crate::assets::AssetManager;
use crate::events::repaint::GlobalIdMap;
use crate::focus::Hover;
use crate::pile::{ImageCard, PendingCards, Pile};
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::math::Rot2;
use bevy::prelude::{
    Component, ComputedNode, Event, KeyCode, Message, Node, Transform, Visibility,
};
use bevy::ui::{UiTransform, Val2};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::PopulatedMessageReader;
use bevy_ecs::observer::On;
use bevy_ecs::query::{Or, With};
use bevy_ecs::system::{Commands, Query, Res, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Message)]
pub struct RotateUi {
    pub entity: Entity,
    pub right: bool,
}
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
    let mut ent = commands.spawn((
        AltMenu {
            entity: event.entity,
        },
        Node { ..Node::default() },
        ImageCard {
            id: card.data.id,
            quality: card.quality,
            transformed: card.transformed,
            global_id: card.global_id,
        },
        card.image_node(assets.card.back_image.clone()),
    ));
    if card.face_handles().is_none() {
        ent.insert(PendingCards);
    }
    if card.ui_rotated() {
        ent.insert(Visibility::Hidden);
        let entity = ent.id();
        commands.write_message(RotateUi {
            entity,
            right: true,
        });
    }
}
#[query_fn]
pub fn on_ui_rotate(
    mut messeges: PopulatedMessageReader<RotateUi>,
    mut query: Query<(&ComputedNode, &mut UiTransform, &mut Visibility)>,
) {
    for event in messeges.read() {
        let mut node = query.get_mut(event.entity).unwrap();
        *node.visibility = Visibility::Inherited;
        node.ui_transform.rotation *= Rot2::from_sin_cos(1.0, 0.0);
        node.ui_transform.translation =
            if matches!(node.ui_transform.rotation.sin_cos(), (1.0 | -1.0, 0.0)) {
                let size = node.computed_node.content_size();
                Val2::px((size.y - size.x) / 2.0, (size.x - size.y) / 2.0)
            } else {
                Val2::px(0.0, 0.0)
            }
    }
}
pub fn on_remove_alt_menu(
    _: On<RemoveAltMenu>,
    ent: Single<Entity, With<AltMenu>>,
    mut commands: Commands,
) {
    commands.entity(*ent).despawn();
}
