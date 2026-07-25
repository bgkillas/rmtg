use crate::events::clipboard::{PollClipboard, get_clipboard};
use crate::events::move_up::move_up;
use crate::paste::react_paste_card;
use bevy::app::App;
pub mod clipboard;
pub mod move_up;
pub fn add_events(app: &mut App) {
    app.add_observer(move_up);
    app.add_observer(get_clipboard);
    app.add_observer(react_paste_card);
    app.init_resource::<PollClipboard>();
}
