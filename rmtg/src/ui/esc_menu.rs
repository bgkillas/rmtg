use crate::ALPN;
use crate::events::clipboard::{ClipboardEvent, GetClipboard};
use crate::keybinds::Keybind;
use crate::net::Msg;
use crate::spatial::Spatial;
use crate::ui::buttons::button;
use crate::ui::menu::{Menu, SetMenu};
use crate::ui::right_click::{RemoveRightClickMenu, RightClickMenu};
use bevy::app::AppExit;
use bevy::clipboard::Clipboard;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::log::warn;
use bevy::prelude::{BackgroundColor, Component, FlexDirection, Resource, Visibility};
use bevy::ui::{Node, PositionType, Val};
use bevy::ui_widgets::{Activate, observe};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::{Single, With};
use bevy_ecs::system::{Commands, If, Res, ResMut};
use bevy_p2p::events::Binded;
use bevy_p2p::iroh_res::{IrohBind, IrohResource, IrohUnbind};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct EscMenu;
#[derive(Component)]
pub struct Exit;
impl EscMenu {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            Self,
            Visibility::Hidden,
            BackgroundColor(Color::srgba_u8(0, 0, 0, 128)),
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.0,
                    width: Val::Percent(25.0),
                    height: Val::Percent(50.0),
                    left: Val::Percent(37.5),
                    top: Val::Percent(25.0),
                    position_type: PositionType::Absolute,
                    ..Node::default()
                },
                Visibility::Inherited,
                children![
                    (button("Copy Endpoint"), observe(on_copy)),
                    (button("Connect To Clipboard"), observe(on_connect)),
                    (button("Disconnect"), observe(on_disconnect)),
                    (button("Moxfield Deck List"), observe(on_moxfield_deck_list)),
                    (button("Import Deck"), observe(on_deck_import)),
                    (button("Exit"), observe(on_exit)),
                ]
            )],
        )
    }
}
#[derive(Resource, Default)]
pub struct CopyOnSpawn;
fn on_copy(
    _: On<Activate>,
    mut commands: Commands,
    iroh: Option<Res<IrohResource<Msg>>>,
    tried: Option<Res<CopyOnSpawn>>,
    mut clipboard: ResMut<Clipboard>,
) {
    if let Some(inner) = iroh {
        if let Err(e) = clipboard.set_text(inner.my_id.to_string()) {
            warn!("{e:?}");
        }
    } else if tried.is_none() {
        commands.init_resource::<CopyOnSpawn>();
        commands.trigger(IrohBind::new(ALPN));
    }
}
fn on_moxfield_deck_list(_: On<Activate>, mut commands: Commands) {
    commands.trigger(SetMenu::new(Menu::Moxfield));
}
fn on_deck_import(_: On<Activate>, mut commands: Commands, spatial: Spatial) {
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    commands.trigger(GetClipboard::text(ClipboardEvent::ImportDeck(pos)));
}
pub fn on_iroh_bind_copy(
    _: On<Binded>,
    _: If<Res<CopyOnSpawn>>,
    mut commands: Commands,
    iroh: Res<IrohResource<Msg>>,
    mut clipboard: ResMut<Clipboard>,
) {
    commands.remove_resource::<CopyOnSpawn>();
    if let Err(e) = clipboard.set_text(iroh.my_id.to_string()) {
        warn!("{e:?}");
    }
}
fn on_connect(_: On<Activate>, mut commands: Commands) {
    commands.trigger(GetClipboard::text(ClipboardEvent::ConnectToEndpoint));
}
fn on_disconnect(_: On<Activate>, mut commands: Commands) {
    commands.trigger(IrohUnbind);
}
fn on_exit(_: On<Activate>, mut writer: MessageWriter<AppExit>) {
    writer.write(AppExit::Success);
}
#[query_fn]
pub fn toggle_esc_menu(
    keybinds: Res<ButtonInput<Keybind>>,
    menu: Res<Menu>,
    mut commands: Commands,
    right_menu: Option<Single<(), With<RightClickMenu>>>,
) {
    if keybinds.just_pressed(Keybind::Menu) {
        if right_menu.is_some() {
            commands.trigger(RemoveRightClickMenu);
        }
        commands.trigger(SetMenu::new(match *menu {
            Menu::World => Menu::Esc,
            Menu::Side | Menu::Counter | Menu::Moxfield | Menu::Esc => Menu::World,
        }));
    }
}
