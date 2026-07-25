use bevy::clipboard::{Clipboard, ClipboardError, ClipboardRead};
use bevy::image::Image;
use bevy::prelude::{Commands, Event, On, ResMut, Resource};
use std::mem;
use std::sync::{Arc, Mutex};
#[derive(Event)]
pub struct GetClipboard {
    pub ty: ClipboardType,
    pub event: ClipboardEvent,
}
#[derive(Event)]
pub struct GotClipboard {
    pub data: ClipboardData,
    pub event: ClipboardEvent,
}
impl GetClipboard {
    #[must_use]
    pub fn text(event: ClipboardEvent) -> Self {
        Self {
            ty: ClipboardType::Text,
            event,
        }
    }
    #[must_use]
    pub fn image(event: ClipboardEvent) -> Self {
        Self {
            ty: ClipboardType::Image,
            event,
        }
    }
}
pub enum ClipboardType {
    Text,
    Image,
}
pub enum ClipboardData {
    Text(String),
    Image(Image),
}
#[derive(Clone, Copy)]
pub enum ClipboardEvent {
    CardSpawn,
}
#[derive(Default, Resource)]
pub struct PollClipboard {
    pub text: Vec<(
        Arc<Mutex<Option<Result<String, ClipboardError>>>>,
        ClipboardEvent,
    )>,
}
pub fn get_clipboard(
    on: On<GetClipboard>,
    mut clipboard: ResMut<Clipboard>,
    mut commands: Commands,
    mut polls: ResMut<PollClipboard>,
) {
    match on.ty {
        ClipboardType::Text => {
            let fetch = clipboard.fetch_text();
            match fetch {
                ClipboardRead::Ready(maybe_value) => {
                    if let Ok(value) = maybe_value {
                        commands.trigger(GotClipboard {
                            data: ClipboardData::Text(value),
                            event: on.event,
                        });
                    }
                }
                ClipboardRead::Pending(poll) => polls.text.push((poll, on.event)),
                ClipboardRead::Taken => unreachable!(),
            }
        }
        ClipboardType::Image => {
            if let Ok(image) = clipboard.fetch_image() {
                commands.trigger(GotClipboard {
                    data: ClipboardData::Image(image),
                    event: on.event,
                });
            }
        }
    }
}
pub fn poll_clipboards(mut polls: ResMut<PollClipboard>, mut commands: Commands) {
    polls.text.retain_mut(|&mut (ref mut poll, event)| {
        if let Some(inner) = &mut *poll.lock().unwrap() {
            if let Ok(value) = inner {
                commands.trigger(GotClipboard {
                    data: ClipboardData::Text(mem::take(value)),
                    event,
                });
            }
            false
        } else {
            true
        }
    });
}
