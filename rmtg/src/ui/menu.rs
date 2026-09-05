use crate::ui::calc::CalcMenu;
use crate::ui::chat::TextMenu;
use crate::ui::esc_menu::EscMenu;
use crate::ui::moxfield::MoxfieldMenu;
use crate::ui::side::{SearchList, SideMenu, SideMenuText};
use crate::ui::text_box::TextSource;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::{Event, Resource, Visibility, Window};
use bevy::text::EditableText;
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::{Query, Single, With};
use bevy_ecs::query::Without;
use bevy_ecs::system::Commands;
use bevy_query_fn_macro::query_fn;
use enumset::{EnumSet, EnumSetType, enum_set};
#[derive(Resource, EnumSetType, Default, Debug)]
pub enum Menu {
    #[default]
    World,
    Moxfield,
    Counter,
    Esc,
    Side,
}
impl Menu {
    pub fn in_view_world(self) -> bool {
        Self::view_world().contains(self)
    }
    pub fn view_world() -> EnumSet<Self> {
        enum_set!(Menu::World | Menu::Moxfield | Menu::Counter | Menu::Side)
    }
}
#[derive(Event)]
pub struct SetMenu {
    pub menu: Menu,
}
impl SetMenu {
    pub fn new(menu: Menu) -> Self {
        Self { menu }
    }
}
#[query_fn]
pub fn on_set_menu(
    event: On<SetMenu>,
    mut esc: Single<
        (Entity, &mut Visibility),
        (
            With<EscMenu>,
            Without<CalcMenu>,
            Without<MoxfieldMenu>,
            Without<SideMenu>,
            Without<TextMenu>,
        ),
    >,
    mut counter: Single<
        (Entity, &mut Visibility),
        (
            With<CalcMenu>,
            Without<EscMenu>,
            Without<MoxfieldMenu>,
            Without<SideMenu>,
            Without<TextMenu>,
        ),
    >,
    mut moxfield: Single<
        (Entity, &mut Visibility),
        (
            With<MoxfieldMenu>,
            Without<CalcMenu>,
            Without<EscMenu>,
            Without<SideMenu>,
            Without<TextMenu>,
        ),
    >,
    mut side: Single<
        (Entity, &mut Visibility),
        (
            With<SideMenu>,
            Without<CalcMenu>,
            Without<MoxfieldMenu>,
            Without<EscMenu>,
            Without<TextMenu>,
        ),
    >,
    mut chat: Single<
        (Entity, &mut Visibility),
        (
            With<TextMenu>,
            Without<SideMenu>,
            Without<CalcMenu>,
            Without<MoxfieldMenu>,
            Without<EscMenu>,
        ),
    >,
    mut menu: ResMut<Menu>,
    mut active_input: ResMut<InputFocus>,
    window: Single<Entity, With<Window>>,
    text_input: Query<(Entity, &TextSource)>,
    mut search_list: Single<(Entity, &mut SearchList)>,
    mut search: Single<&mut EditableText, With<SideMenuText>>,
    mut commands: Commands,
) {
    match *menu {
        Menu::World => {}
        Menu::Side => {
            search.clear();
            commands.entity(search_list.entity).despawn_children();
            search_list.search_list.list = None;
            *side.visibility = Visibility::Hidden;
        }
        Menu::Counter => *counter.visibility = Visibility::Hidden,
        Menu::Moxfield => {
            *chat.visibility = Visibility::Visible;
            *moxfield.visibility = Visibility::Hidden;
        }
        Menu::Esc => *esc.visibility = Visibility::Hidden,
    }
    let ent = match event.menu {
        Menu::World => *window,
        Menu::Side => {
            *side.visibility = Visibility::Visible;
            text_input
                .iter()
                .find(|q| matches!(q.text_source, TextSource::Search))
                .unwrap()
                .entity
        }
        Menu::Counter => {
            *counter.visibility = Visibility::Visible;
            counter.entity
        }
        Menu::Moxfield => {
            *chat.visibility = Visibility::Hidden;
            *moxfield.visibility = Visibility::Visible;
            text_input
                .iter()
                .find(|q| matches!(q.text_source, TextSource::Moxfield))
                .unwrap()
                .entity
        }
        Menu::Esc => {
            *esc.visibility = Visibility::Visible;
            esc.entity
        }
    };
    active_input.set(ent, FocusCause::Pressed);
    *menu = event.menu;
}
