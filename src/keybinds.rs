use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::prelude::{Deref, DerefMut, KeyCode, MouseButton, Res, ResMut, Resource};
use enum_map::{Enum, EnumMap, enum_map};
use enumset::{EnumSet, EnumSetType, enum_set};
use std::fmt;
use std::fmt::{Display, Formatter};
#[derive(SystemParam)]
pub struct Keybinds<'w> {
    pub keyboard: Res<'w, ButtonInput<KeyCode>>,
    pub mouse: Res<'w, ButtonInput<MouseButton>>,
    pub keybinds: ResMut<'w, KeybindsList>,
}
impl Keybinds<'_> {
    pub fn just_pressed(&self, keybind: Keybind) -> bool {
        self.keybinds[keybind].just_pressed(&self.keyboard, &self.mouse)
    }
    pub fn pressed(&self, keybind: Keybind) -> bool {
        self.keybinds[keybind].pressed(&self.keyboard, &self.mouse)
    }
    pub fn get_numeric(&self) -> usize {
        match DIGITS.iter().find(|n| self.keyboard.pressed(**n)) {
            Some(KeyCode::Digit0) | Some(KeyCode::Numpad0) => 0,
            Some(KeyCode::Digit1) | Some(KeyCode::Numpad1) => 1,
            Some(KeyCode::Digit2) | Some(KeyCode::Numpad2) => 2,
            Some(KeyCode::Digit3) | Some(KeyCode::Numpad3) => 3,
            Some(KeyCode::Digit4) | Some(KeyCode::Numpad4) => 4,
            Some(KeyCode::Digit5) | Some(KeyCode::Numpad5) => 5,
            Some(KeyCode::Digit6) | Some(KeyCode::Numpad6) => 6,
            Some(KeyCode::Digit7) | Some(KeyCode::Numpad7) => 7,
            Some(KeyCode::Digit8) | Some(KeyCode::Numpad8) => 8,
            Some(KeyCode::Digit9) | Some(KeyCode::Numpad9) => 9,
            _ => unreachable!(),
        }
    }
    #[allow(dead_code)]
    pub fn set(&mut self, keybind: Keybind) -> bool {
        if let Some(new) = Bind::new_from(&self.keyboard, &self.mouse) {
            self.keybinds[keybind] = new;
            true
        } else {
            false
        }
    }
}
#[derive(Enum, Debug)]
pub enum Keybind {
    Ping,
    HostSteam,
    HostIp,
    JoinIp,
    SortHand,
    Select,
    Flip,
    Shuffle,
    Remove,
    Copy,
    CopyObject,
    Paste,
    PasteObject,
    PickCard,
    Equip,
    RotateRight,
    RotateLeft,
    Spread,
    Printings,
    Tokens,
    Transform,
    Search,
    View,
    Sub,
    Add,
    Calc,
    Chat,
    Voice,
    TakeTurn,
    PassTurn,
    Menu,
    CalcClose,
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
    Untap,
    ScaleUp,
    ScaleDown,
    Mill,
    Exile,
    Reveal,
    Draw,
    Loyalty,
    Power,
    Toughness,
    MiscCounter,
    Counters,
}
#[derive(Resource, Deref, DerefMut)]
pub struct KeybindsList(EnumMap<Keybind, Bind>);
impl Default for KeybindsList {
    fn default() -> Self {
        let ctrl = Modifier::Control;
        let alt = Modifier::Alt;
        let shift = Modifier::Shift;
        Self(enum_map! {
            Keybind::Ping => Bind::new(enum_set!(), MouseButton::Middle),
            Keybind::Select => Bind::new(enum_set!(), MouseButton::Left),
            Keybind::Add => Bind::new(enum_set!(), MouseButton::Left),
            Keybind::Sub => Bind::new(enum_set!(), MouseButton::Right),
            Keybind::PickCard => Bind::new(enum_set!(ctrl), MouseButton::Left),
            Keybind::HostSteam => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyN),
            Keybind::HostIp => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyM),
            Keybind::JoinIp => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyK),
            Keybind::SortHand => Bind::new(enum_set!(ctrl), KeyCode::KeyS),
            Keybind::Flip => Bind::new(enum_set!(), KeyCode::KeyF),
            Keybind::Shuffle => Bind::new(enum_set!(), KeyCode::KeyR),
            Keybind::Calc => Bind::new(enum_set!(ctrl), KeyCode::KeyR),
            Keybind::Remove => Bind::new(enum_set!(), KeyCode::Delete),
            Keybind::Copy => Bind::new(enum_set!(ctrl), KeyCode::KeyC),
            Keybind::CopyObject => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyC),
            Keybind::Paste => Bind::new(enum_set!(ctrl), KeyCode::KeyV),
            Keybind::PasteObject => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyV),
            Keybind::Equip => Bind::new(enum_set!(ctrl), KeyCode::KeyE),
            Keybind::RotateLeft => Bind::new(enum_set!(), KeyCode::KeyQ),
            Keybind::RotateRight => Bind::new(enum_set!(), KeyCode::KeyE),
            Keybind::Spread => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyS),
            Keybind::Printings => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyO),
            Keybind::Tokens => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyT),
            Keybind::Transform => Bind::new(enum_set!(), KeyCode::KeyO),
            Keybind::Search => Bind::new(enum_set!(ctrl), KeyCode::KeyZ),
            Keybind::View => Bind::new(enum_set!(alt), Key::None),
            Keybind::Chat => Bind::new(enum_set!(), KeyCode::Enter),
            Keybind::Voice => Bind::new(enum_set!(), KeyCode::KeyB),
            Keybind::TakeTurn => Bind::new(enum_set!(ctrl), KeyCode::KeyX),
            Keybind::PassTurn => Bind::new(enum_set!(), KeyCode::KeyX),
            Keybind::Menu => Bind::new(enum_set!(), KeyCode::Escape),
            Keybind::CalcClose => Bind::new(enum_set!(), KeyCode::Enter),
            Keybind::Left => Bind::new(enum_set!(), KeyCode::KeyA),
            Keybind::Up => Bind::new(enum_set!(), KeyCode::KeyW),
            Keybind::Down => Bind::new(enum_set!(), KeyCode::KeyS),
            Keybind::Right => Bind::new(enum_set!(), KeyCode::KeyD),
            Keybind::LeftFast => Bind::new(enum_set!(shift), KeyCode::KeyA),
            Keybind::UpFast => Bind::new(enum_set!(shift), KeyCode::KeyW),
            Keybind::DownFast => Bind::new(enum_set!(shift), KeyCode::KeyS),
            Keybind::RightFast => Bind::new(enum_set!(shift), KeyCode::KeyD),
            Keybind::Reset => Bind::new(enum_set!(), KeyCode::Space),
            Keybind::Rotate => Bind::new(enum_set!(), MouseButton::Right),
            Keybind::Untap => Bind::new(enum_set!(), KeyCode::KeyU),
            Keybind::ScaleUp => Bind::new(enum_set!(), KeyCode::Equal),
            Keybind::ScaleDown => Bind::new(enum_set!(), KeyCode::Minus),
            Keybind::Mill => Bind::new(enum_set!(ctrl), Key::Numeric),
            Keybind::Exile => Bind::new(enum_set!(ctrl | shift), Key::Numeric),
            Keybind::Reveal => Bind::new(enum_set!(alt), Key::Numeric),
            Keybind::Draw => Bind::new(enum_set!(), Key::Numeric),
            Keybind::Loyalty => Bind::new(enum_set!(alt), KeyCode::KeyL),
            Keybind::Power => Bind::new(enum_set!(alt), KeyCode::KeyP),
            Keybind::Toughness => Bind::new(enum_set!(alt), KeyCode::KeyT),
            Keybind::MiscCounter => Bind::new(enum_set!(alt), KeyCode::KeyM),
            Keybind::Counters => Bind::new(enum_set!(alt), KeyCode::KeyC),
        })
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
        write!(
            f,
            "{}{:?}",
            self.modifiers
                .iter()
                .map(|m| format!("{m:?}+"))
                .collect::<Vec<String>>()
                .join(""),
            self.key
        )
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
    pub fn pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.any_pressed(match self {
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        })
    }
    #[allow(dead_code)]
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
pub struct Bind {
    modifiers: EnumSet<Modifier>,
    key: Key,
}
impl From<KeyCode> for Bind {
    fn from(value: KeyCode) -> Self {
        Self {
            modifiers: EnumSet::default(),
            key: value.into(),
        }
    }
}
impl From<MouseButton> for Bind {
    fn from(value: MouseButton) -> Self {
        Self {
            modifiers: EnumSet::default(),
            key: value.into(),
        }
    }
}
impl Bind {
    #[allow(dead_code)]
    pub fn new_from(
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> Option<Self> {
        let mut modifiers = EnumSet::empty();
        for modifier in keyboard.get_pressed().flat_map(|k| k.try_into().ok()) {
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
            Some(Self {
                modifiers,
                key: key.into(),
            })
        } else if let Some(key) = keyboard.copied() {
            if keyboard_pressed.next().is_some() {
                return None;
            }
            Some(Self {
                modifiers,
                key: key.into(),
            })
        } else {
            None
        }
    }
    pub fn new(modifiers: EnumSet<Modifier>, key: impl Into<Key>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }
    pub fn modifiers_pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        self.modifiers.iter().all(|m| m.pressed(keyboard))
        /*&& keyboard.get_pressed().all(|k| {
            if let Ok(m) = k.try_into() {
                self.modifiers.contains(m)
            } else {
                true
            }
        })*/
    }
    pub fn just_pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        (match self.key {
            Key::KeyCode(key) => keyboard.just_pressed(key),
            Key::Mouse(button) => mouse.just_pressed(button),
            Key::None => self.modifiers.iter().all(|m| m.just_pressed(keyboard)),
            Key::Numeric => DIGITS.iter().any(|n| keyboard.just_pressed(*n)),
        }) && self.modifiers_pressed(keyboard)
    }
    pub fn pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        (match self.key {
            Key::KeyCode(key) => keyboard.pressed(key),
            Key::Mouse(button) => mouse.pressed(button),
            Key::None => true,
            Key::Numeric => DIGITS.iter().any(|n| keyboard.pressed(*n)),
        }) && self.modifiers_pressed(keyboard)
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
