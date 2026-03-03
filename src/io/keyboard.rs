use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalKey(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Space,
    Return,
    Escape,
    Backspace,
    Tab,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadEnter,
    Other(u32),
}

impl KeyCode {
    pub fn from_name(name: &str) -> Option<Self> {
        use KeyCode::*;
        Some(match name.to_lowercase().as_str() {
            "a" => A,
            "b" => B,
            "c" => C,
            "d" => D,
            "e" => E,
            "f" => F,
            "g" => G,
            "h" => H,
            "i" => I,
            "j" => J,
            "k" => K,
            "l" => L,
            "m" => M,
            "n" => N,
            "o" => O,
            "p" => P,
            "q" => Q,
            "r" => R,
            "s" => S,
            "t" => T,
            "u" => U,
            "v" => V,
            "w" => W,
            "x" => X,
            "y" => Y,
            "z" => Z,
            "0" => Key0,
            "1" => Key1,
            "2" => Key2,
            "3" => Key3,
            "4" => Key4,
            "5" => Key5,
            "6" => Key6,
            "7" => Key7,
            "8" => Key8,
            "9" => Key9,
            "f1" => F1,
            "f2" => F2,
            "f3" => F3,
            "f4" => F4,
            "f5" => F5,
            "f6" => F6,
            "f7" => F7,
            "f8" => F8,
            "f9" => F9,
            "f10" => F10,
            "f11" => F11,
            "f12" => F12,
            "space" | " " => Space,
            "return" | "enter" => Return,
            "escape" | "esc" => Escape,
            "backspace" => Backspace,
            "tab" => Tab,
            "left" | "leftarrow" => LeftArrow,
            "right" | "rightarrow" => RightArrow,
            "up" | "uparrow" => UpArrow,
            "down" | "downarrow" => DownArrow,
            _ => return None,
        })
    }

    pub fn as_name(&self) -> &'static str {
        use KeyCode::*;
        match self {
            A => "a",
            B => "b",
            C => "c",
            D => "d",
            E => "e",
            F => "f",
            G => "g",
            H => "h",
            I => "i",
            J => "j",
            K => "k",
            L => "l",
            M => "m",
            N => "n",
            O => "o",
            P => "p",
            Q => "q",
            R => "r",
            S => "s",
            T => "t",
            U => "u",
            V => "v",
            W => "w",
            X => "x",
            Y => "y",
            Z => "z",
            Key0 => "0",
            Key1 => "1",
            Key2 => "2",
            Key3 => "3",
            Key4 => "4",
            Key5 => "5",
            Key6 => "6",
            Key7 => "7",
            Key8 => "8",
            Key9 => "9",
            F1 => "f1",
            F2 => "f2",
            F3 => "f3",
            F4 => "f4",
            F5 => "f5",
            F6 => "f6",
            F7 => "f7",
            F8 => "f8",
            F9 => "f9",
            F10 => "f10",
            F11 => "f11",
            F12 => "f12",
            Space => "space",
            Return => "return",
            Escape => "escape",
            Backspace => "backspace",
            Tab => "tab",
            LeftArrow => "left",
            RightArrow => "right",
            UpArrow => "up",
            DownArrow => "down",
            LeftShift => "lshift",
            RightShift => "rshift",
            LeftCtrl => "lctrl",
            RightCtrl => "rctrl",
            LeftAlt => "lalt",
            RightAlt => "ralt",
            Numpad0 => "num0",
            Numpad1 => "num1",
            Numpad2 => "num2",
            Numpad3 => "num3",
            Numpad4 => "num4",
            Numpad5 => "num5",
            Numpad6 => "num6",
            Numpad7 => "num7",
            Numpad8 => "num8",
            Numpad9 => "num9",
            NumpadEnter => "numenter",
            Other(_) => "unknown",
        }
    }

    pub fn is_modifier(&self) -> bool {
        use KeyCode::*;
        matches!(
            self,
            LeftShift | RightShift | LeftCtrl | RightCtrl | LeftAlt | RightAlt
        )
    }

    /// Convert a winit logical key to our KeyCode.
    pub fn from_winit(key: &winit::keyboard::Key) -> Option<Self> {
        use winit::keyboard::{Key, NamedKey};
        Some(match key {
            Key::Character(s) => match s.as_str() {
                "a" | "A" => Self::A,
                "b" | "B" => Self::B,
                "c" | "C" => Self::C,
                "d" | "D" => Self::D,
                "e" | "E" => Self::E,
                "f" | "F" => Self::F,
                "g" | "G" => Self::G,
                "h" | "H" => Self::H,
                "i" | "I" => Self::I,
                "j" | "J" => Self::J,
                "k" | "K" => Self::K,
                "l" | "L" => Self::L,
                "m" | "M" => Self::M,
                "n" | "N" => Self::N,
                "o" | "O" => Self::O,
                "p" | "P" => Self::P,
                "q" | "Q" => Self::Q,
                "r" | "R" => Self::R,
                "s" | "S" => Self::S,
                "t" | "T" => Self::T,
                "u" | "U" => Self::U,
                "v" | "V" => Self::V,
                "w" | "W" => Self::W,
                "x" | "X" => Self::X,
                "y" | "Y" => Self::Y,
                "z" | "Z" => Self::Z,
                "0" => Self::Key0,
                "1" => Self::Key1,
                "2" => Self::Key2,
                "3" => Self::Key3,
                "4" => Self::Key4,
                "5" => Self::Key5,
                "6" => Self::Key6,
                "7" => Self::Key7,
                "8" => Self::Key8,
                "9" => Self::Key9,
                _ => return None,
            },
            Key::Named(n) => match n {
                NamedKey::Space => Self::Space,
                NamedKey::Enter => Self::Return,
                NamedKey::Escape => Self::Escape,
                NamedKey::Backspace => Self::Backspace,
                NamedKey::Tab => Self::Tab,
                NamedKey::ArrowLeft => Self::LeftArrow,
                NamedKey::ArrowRight => Self::RightArrow,
                NamedKey::ArrowUp => Self::UpArrow,
                NamedKey::ArrowDown => Self::DownArrow,
                NamedKey::Shift => Self::LeftShift,
                NamedKey::Control => Self::LeftCtrl,
                NamedKey::Alt => Self::LeftAlt,
                NamedKey::F1 => Self::F1,
                NamedKey::F2 => Self::F2,
                NamedKey::F3 => Self::F3,
                NamedKey::F4 => Self::F4,
                NamedKey::F5 => Self::F5,
                NamedKey::F6 => Self::F6,
                NamedKey::F7 => Self::F7,
                NamedKey::F8 => Self::F8,
                NamedKey::F9 => Self::F9,
                NamedKey::F10 => Self::F10,
                NamedKey::F11 => Self::F11,
                NamedKey::F12 => Self::F12,
                _ => return None,
            },
            _ => return None,
        })
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_name())
    }
}

static KEY_QUEUE: Mutex<VecDeque<(KeyCode, KeyState)>> = Mutex::new(VecDeque::new());

/// Called by the winit event handler on the render thread.
pub fn push_key_event(code: KeyCode, state: KeyState) {
    if let Ok(mut q) = KEY_QUEUE.lock() {
        q.push_back((code, state));
    }
}

/// Called by ResponseWindow on the script thread.
pub fn poll_key_event() -> Option<(KeyCode, KeyState)> {
    KEY_QUEUE.lock().ok()?.pop_front()
}

/// Discard all pending key events
pub fn flush_key_buffer() {
    if let Ok(mut q) = KEY_QUEUE.lock() {
        q.clear();
    }
}
