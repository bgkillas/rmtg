use crate::events::move_up::move_up;
use bevy::app::App;
pub mod move_up;
pub fn add_events(app: &mut App) {
    app.add_observer(move_up);
}
