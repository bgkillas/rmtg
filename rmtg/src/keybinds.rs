#![expect(clippy::shadow_reuse)]
use crate::focus::Focus;
use crate::ui::menu::Menu;
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::prelude::{Deref, DerefMut, KeyCode, MouseButton, Res, ResMut, Resource};
use enum_map::{Enum, EnumMap, enum_map};
use enumset::{EnumSet, EnumSetType, enum_set};
use std::fmt;
use std::fmt::{Display, Formatter};
#[derive(SystemParam)]
pub struct Keybinds<'w, 's> {
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub keybinds: ResMut<'w, KeybindsList>,
    pub focus: Focus<'w, 's>,
}
pub fn update_keybinds(mut keybind_input: ResMut<ButtonInput<Keybind>>, keybinds: Keybinds) {
    for keybind in EnumSet::all() {
        match (keybinds.pressed(keybind), keybind_input.pressed(keybind)) {
            (true, true) | (false, false) => {
                keybind_input.clear_just_pressed(keybind);
                keybind_input.clear_just_released(keybind);
            }
            (true, false) => keybind_input.press(keybind),
            (false, true) => keybind_input.release(keybind),
        }
    }
}
impl Keybinds<'_, '_> {
    #[must_use]
    pub fn just_pressed(&self, keybind: Keybind) -> bool {
        self.keybinds[keybind].just_pressed(&self.keyboard, &self.mouse)
    }
    #[must_use]
    pub fn pressed(&self, keybind: Keybind) -> bool {
        self.keybinds[keybind].pressed(&self.keyboard, &self.mouse, &self.focus)
    }
    #[must_use]
    pub fn get_numeric(&self) -> usize {
        match DIGITS.iter().find(|n| self.keyboard.pressed(**n)) {
            Some(KeyCode::Digit0 | KeyCode::Numpad0) => 0,
            Some(KeyCode::Digit1 | KeyCode::Numpad1) => 1,
            Some(KeyCode::Digit2 | KeyCode::Numpad2) => 2,
            Some(KeyCode::Digit3 | KeyCode::Numpad3) => 3,
            Some(KeyCode::Digit4 | KeyCode::Numpad4) => 4,
            Some(KeyCode::Digit5 | KeyCode::Numpad5) => 5,
            Some(KeyCode::Digit6 | KeyCode::Numpad6) => 6,
            Some(KeyCode::Digit7 | KeyCode::Numpad7) => 7,
            Some(KeyCode::Digit8 | KeyCode::Numpad8) => 8,
            Some(KeyCode::Digit9 | KeyCode::Numpad9) => 9,
            _ => unreachable!(),
        }
    }
    #[must_use]
    pub fn set(&mut self, keybind: Keybind) -> bool {
        if let Some(new) = Bind::new_from(&self.keyboard, &self.mouse) {
            self.keybinds[keybind] = new;
            true
        } else {
            false
        }
    }
}
#[derive(Enum, EnumSetType, Debug, Hash)]
pub enum Keybind {
    Select,
    HoldSelect,
    Shuffle,
    Remove,
    CopyObject,
    PasteObject,
    Chat,
    Menu,
    Left,
    Right,
    Up,
    Down,
    LeftFast,
    RightFast,
    UpFast,
    DownFast,
    Reset,
    Rotate,
    ScaleUp,
    ScaleDown,
}
#[derive(Resource, Deref, DerefMut)]
pub struct KeybindsList(EnumMap<Keybind, Bind>);
impl Default for KeybindsList {
    fn default() -> Self {
        let ctrl = Modifier::Control;
        let alt = Modifier::Alt;
        _ = alt;
        let shift = Modifier::Shift;
        let map = enum_map! {
            Keybind::Select =>      Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  MouseButton::Left),
            Keybind::HoldSelect =>  Bind::new(enum_set!(ctrl),  Menu::view_world(),         false, true,  MouseButton::Left),
            Keybind::Rotate =>      Bind::new(enum_set!(),      Menu::view_world(),         false, true,  MouseButton::Right),
            Keybind::Shuffle =>     Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::KeyR),
            Keybind::Remove =>      Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::Delete),
            Keybind::CopyObject =>  Bind::new(enum_set!(ctrl),  Menu::view_world(),         true,  true,  KeyCode::KeyC),
            Keybind::PasteObject => Bind::new(enum_set!(ctrl),  Menu::view_world(),         true,  true,  KeyCode::KeyV),
            Keybind::Chat =>        Bind::new(enum_set!(),      enum_set!(Menu::World),     true,  false, KeyCode::Enter),
            Keybind::Menu =>        Bind::new(enum_set!(),      EnumSet::all(),             true,  false, KeyCode::Escape),
            Keybind::Left =>        Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::KeyA),
            Keybind::Up =>          Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::KeyW),
            Keybind::Down =>        Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::KeyS),
            Keybind::Right =>       Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::KeyD),
            Keybind::LeftFast =>    Bind::new(enum_set!(shift), Menu::view_world(),         true,  true,  KeyCode::KeyA),
            Keybind::UpFast =>      Bind::new(enum_set!(shift), Menu::view_world(),         true,  true,  KeyCode::KeyW),
            Keybind::DownFast =>    Bind::new(enum_set!(shift), Menu::view_world(),         true,  true,  KeyCode::KeyS),
            Keybind::RightFast =>   Bind::new(enum_set!(shift), Menu::view_world(),         true,  true,  KeyCode::KeyD),
            Keybind::Reset =>       Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::Space),
            Keybind::ScaleUp =>     Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::Equal),
            Keybind::ScaleDown =>   Bind::new(enum_set!(),      Menu::view_world(),         true,  true,  KeyCode::Minus),
        };
        Self(map)
    }
}
impl Display for KeybindsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|a| format!("{:?} => {}", a.0, a.1))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
impl Display for Bind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for m in self.modifiers.iter() {
            write!(f, "{m:?}+")?;
        }
        write!(f, "{:?}", self.key)
    }
}
#[derive(PartialEq, Debug)]
pub enum Key {
    KeyCode(KeyCode),
    Mouse(MouseButton),
    Numeric,
    None,
}
impl From<KeyCode> for Key {
    fn from(value: KeyCode) -> Self {
        Self::KeyCode(value)
    }
}
impl From<MouseButton> for Key {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}
#[derive(EnumSetType, Debug)]
pub enum Modifier {
    Alt,
    Control,
    Shift,
    Super,
}
impl Modifier {
    #[must_use]
    pub fn pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.any_pressed(match self {
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        })
    }
    #[must_use]
    pub fn just_pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.any_just_pressed(match self {
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        })
    }
}
impl TryFrom<&KeyCode> for Modifier {
    type Error = ();
    fn try_from(value: &KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::AltLeft | KeyCode::AltRight => Modifier::Alt,
            KeyCode::ControlLeft | KeyCode::ControlRight => Modifier::Control,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Modifier::Shift,
            KeyCode::SuperLeft | KeyCode::SuperRight => Modifier::Super,
            _ => return Err(()),
        })
    }
}
#[derive(Debug)]
pub struct Bind {
    modifiers: EnumSet<Modifier>,
    key: Key,
    strict: bool,
    lock: bool,
    menus: EnumSet<Menu>,
}
impl Bind {
    #[must_use]
    pub fn new_from(
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> Option<Self> {
        let mut modifiers = EnumSet::empty();
        for modifier in keyboard.get_pressed().filter_map(|k| k.try_into().ok()) {
            modifiers.insert(modifier);
        }
        let mut mouse_pressed = mouse.get_just_pressed();
        let mouse = mouse_pressed.next();
        let mut keyboard_pressed = keyboard
            .get_just_pressed()
            .filter(|k| Modifier::try_from(*k).is_err());
        let keyboard = keyboard_pressed.next();
        if let Some(key) = mouse.copied() {
            if mouse_pressed.next().is_some() {
                return None;
            }
            Some(Self::new(modifiers, EnumSet::all(), true, true, key))
        } else if let Some(key) = keyboard.copied() {
            if keyboard_pressed.next().is_some() {
                return None;
            }
            Some(Self::new(modifiers, EnumSet::all(), true, true, key))
        } else {
            None
        }
    }
    #[must_use]
    pub fn new(
        modifiers: EnumSet<Modifier>,
        menus: EnumSet<Menu>,
        strict: bool,
        lock: bool,
        to_key: impl Into<Key>,
    ) -> Self {
        let key = to_key.into();
        Self {
            modifiers,
            key,
            strict,
            lock,
            menus,
        }
    }
    #[must_use]
    pub fn modifiers_pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        self.modifiers.iter().all(|m| m.pressed(keyboard))
            && (!self.strict
                || keyboard.get_pressed().all(|k| {
                    if let Ok(m) = k.try_into() {
                        self.modifiers.contains(m)
                    } else {
                        true
                    }
                }))
    }
    #[must_use]
    pub fn just_pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        self.modifiers_pressed(keyboard)
            && match self.key {
                Key::KeyCode(key) => keyboard.just_pressed(key),
                Key::Mouse(button) => mouse.just_pressed(button),
                Key::None => self.modifiers.iter().all(|m| m.just_pressed(keyboard)),
                Key::Numeric => DIGITS.iter().any(|n| keyboard.just_pressed(*n)),
            }
    }
    #[must_use]
    pub fn pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
        focus: &Focus,
    ) -> bool {
        self.menus.contains(*focus.menu)
            && self.modifiers_pressed(keyboard)
            && match self.key {
                Key::KeyCode(key) => {
                    (!self.lock || !focus.key_lock(EnumSet::all())) && keyboard.pressed(key)
                }
                Key::Mouse(button) => {
                    (!self.lock || !focus.mouse_lock(EnumSet::all())) && mouse.pressed(button)
                }
                Key::None => true,
                Key::Numeric => DIGITS.iter().any(|n| keyboard.pressed(*n)),
            }
    }
}
const DIGITS: [KeyCode; 20] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Numpad0,
    KeyCode::Numpad1,
    KeyCode::Numpad2,
    KeyCode::Numpad3,
    KeyCode::Numpad4,
    KeyCode::Numpad5,
    KeyCode::Numpad6,
    KeyCode::Numpad7,
    KeyCode::Numpad8,
    KeyCode::Numpad9,
];
