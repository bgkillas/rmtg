use crate::paste::react_paste_card;
use bevy::clipboard::{Clipboard, ClipboardError, ClipboardRead};
use bevy::image::Image;
use bevy::log::warn;
use bevy::prelude::{Commands, Event, On, ResMut, Resource};
use std::mem;
use std::sync::{Arc, Mutex};
#[derive(Event)]
pub struct GetClipboard {
    pub ty: ClipboardType,
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
impl ClipboardEvent {
    pub fn run(self, commands: &mut Commands, text: String) {
        match self {
            Self::CardSpawn => {
                commands.run_system_cached_with(react_paste_card, text);
            }
        }
    }
    pub fn run_image(self, commands: &mut Commands, image: Image) {
        _ = commands;
        _ = image;
    }
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
                ClipboardRead::Ready(maybe_value) => match maybe_value {
                    Ok(mut value) => on.event.run(&mut commands, mem::take(&mut value)),
                    Err(e) => warn!("{e:?}"),
                },
                ClipboardRead::Pending(poll) => polls.text.push((poll, on.event)),
                ClipboardRead::Taken => unreachable!(),
            }
        }
        ClipboardType::Image =>
        {
            #[cfg(not(target_family = "wasm"))]
            if let Ok(image) = clipboard.fetch_image() {
                on.event.run_image(&mut commands, image);
            }
        }
    }
}
pub fn poll_clipboards(mut polls: ResMut<PollClipboard>, mut commands: Commands) {
    polls.text.retain_mut(|&mut (ref mut poll, event)| {
        if let Some(inner) = &mut *poll.lock().unwrap() {
            match inner {
                Ok(value) => event.run(&mut commands, mem::take(value)),
                Err(e) => warn!("{e:?}"),
            }
            false
        } else {
            true
        }
    });
}
