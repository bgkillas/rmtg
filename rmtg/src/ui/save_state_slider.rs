use crate::events::save_states::{ApplySaveState, SaveStates};
use crate::ui::menu::{Menu, SetMenu};
use crate::ui::sliders::horizontal_slider;
use bevy::prelude::{Component, Visibility};
use bevy::ui_widgets::{SliderRange, SliderValue, observe};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::With;
use bevy_ecs::system::{Commands, Local, Query, Res, ResMut, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct SaveStateSliderMenu;
impl SaveStateSliderMenu {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            horizontal_slider(0.0, 0.0, 0.0),
            observe(slider_changed),
            Self,
            Visibility::Hidden,
        )
    }
}
#[query_fn]
pub fn on_set_slider_ui(
    event: On<SetMenu>,
    save: Single<Entity, With<SaveStateSliderMenu>>,
    mut commands: Commands,
    mut states: ResMut<SaveStates>,
) {
    if !matches!(event.menu, Menu::SaveStateSlider) {
        return;
    }
    states.pause = true;
    let end = (states.states.len() - 1) as f32;
    commands.entity(*save).insert(SliderRange::new(0.0, end));
    commands.entity(*save).insert(SliderValue(end));
}
pub fn slider_changed(
    event: On<Insert, SliderValue>,
    sliders: Query<&SliderValue>,
    mut commands: Commands,
    mut last: Local<usize>,
    states: Res<SaveStates>,
) {
    let value = sliders.get(event.entity).unwrap();
    let int = value.0.round() as usize;
    if int == *last {
        return;
    }
    *last = int;
    if let Some(state) = states.states.nth_back(int) {
        commands.trigger(ApplySaveState::local(state.clone()));
    }
}
