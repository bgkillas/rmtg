use crate::events::clipboard::{PollClipboard, get_clipboard};
use crate::events::hover::{add_hover, remove_hover};
use crate::events::move_up::move_up;
use crate::events::repaint::on_repaint;
use crate::events::roll::on_roll;
use crate::paste::react_paste_card;
use bevy::app::App;
pub mod clipboard;
pub mod hover;
pub mod move_up;
pub mod repaint;
pub mod roll;
pub fn add_events(app: &mut App) {
    app.add_observer(move_up);
    app.add_observer(get_clipboard);
    app.add_observer(react_paste_card);
    app.add_observer(on_roll);
    app.add_observer(on_repaint);
    app.add_observer(add_hover);
    app.add_observer(remove_hover);
    app.init_resource::<PollClipboard>();
}
