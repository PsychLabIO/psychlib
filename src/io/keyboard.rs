use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    Pressed,
    Released,
}

/// A physical keyboard key, before any layout mapping.
/// Used internally by the platform layer; experiment code works with `KeyCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalKey(pub u32);

/// Luau scripts refer to letter keys by their lowercase character string
/// ("f", "j") which maps to `KeyCode::F`, `KeyCode::J`, etc.
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
            // Letters
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
            // Digits
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
            // Function keys
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
            // Special
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

    /// Return the canonical lowercase name for this key, matching the Luau API.
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
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_name())
    }
}

pub fn poll_key_event() -> Option<(KeyCode, KeyState)> {
    platform::poll_key_event()
}

pub fn flush_key_buffer() {
    platform::flush_key_buffer();
}

mod platform {
    use super::{KeyCode, KeyState};

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_key_event() -> Option<(KeyCode, KeyState)> {
        use crossterm::event::{self, Event, KeyEventKind};
        use std::time::Duration;

        if !event::poll(Duration::ZERO).unwrap_or(false) {
            return None;
        }

        match event::read().ok()? {
            Event::Key(key_event) => {
                let state = match key_event.kind {
                    KeyEventKind::Press => KeyState::Pressed,
                    KeyEventKind::Release => KeyState::Released,
                    KeyEventKind::Repeat => KeyState::Pressed,
                };
                let code = crossterm_code_to_keycode(key_event.code);
                Some((code, state))
            }
            _ => None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn flush_key_buffer() {
        use crossterm::event::{self};
        use std::time::Duration;
        while event::poll(Duration::ZERO).unwrap_or(false) {
            let _ = event::read();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn crossterm_code_to_keycode(code: crossterm::event::KeyCode) -> KeyCode {
        use crossterm::event::KeyCode as CT;
        match code {
            CT::Char('a') | CT::Char('A') => KeyCode::A,
            CT::Char('b') | CT::Char('B') => KeyCode::B,
            CT::Char('c') | CT::Char('C') => KeyCode::C,
            CT::Char('d') | CT::Char('D') => KeyCode::D,
            CT::Char('e') | CT::Char('E') => KeyCode::E,
            CT::Char('f') | CT::Char('F') => KeyCode::F,
            CT::Char('g') | CT::Char('G') => KeyCode::G,
            CT::Char('h') | CT::Char('H') => KeyCode::H,
            CT::Char('i') | CT::Char('I') => KeyCode::I,
            CT::Char('j') | CT::Char('J') => KeyCode::J,
            CT::Char('k') | CT::Char('K') => KeyCode::K,
            CT::Char('l') | CT::Char('L') => KeyCode::L,
            CT::Char('m') | CT::Char('M') => KeyCode::M,
            CT::Char('n') | CT::Char('N') => KeyCode::N,
            CT::Char('o') | CT::Char('O') => KeyCode::O,
            CT::Char('p') | CT::Char('P') => KeyCode::P,
            CT::Char('q') | CT::Char('Q') => KeyCode::Q,
            CT::Char('r') | CT::Char('R') => KeyCode::R,
            CT::Char('s') | CT::Char('S') => KeyCode::S,
            CT::Char('t') | CT::Char('T') => KeyCode::T,
            CT::Char('u') | CT::Char('U') => KeyCode::U,
            CT::Char('v') | CT::Char('V') => KeyCode::V,
            CT::Char('w') | CT::Char('W') => KeyCode::W,
            CT::Char('x') | CT::Char('X') => KeyCode::X,
            CT::Char('y') | CT::Char('Y') => KeyCode::Y,
            CT::Char('z') | CT::Char('Z') => KeyCode::Z,
            CT::Char('0') => KeyCode::Key0,
            CT::Char('1') => KeyCode::Key1,
            CT::Char('2') => KeyCode::Key2,
            CT::Char('3') => KeyCode::Key3,
            CT::Char('4') => KeyCode::Key4,
            CT::Char('5') => KeyCode::Key5,
            CT::Char('6') => KeyCode::Key6,
            CT::Char('7') => KeyCode::Key7,
            CT::Char('8') => KeyCode::Key8,
            CT::Char('9') => KeyCode::Key9,
            CT::F(1) => KeyCode::F1,
            CT::F(2) => KeyCode::F2,
            CT::F(3) => KeyCode::F3,
            CT::F(4) => KeyCode::F4,
            CT::F(5) => KeyCode::F5,
            CT::F(6) => KeyCode::F6,
            CT::F(7) => KeyCode::F7,
            CT::F(8) => KeyCode::F8,
            CT::F(9) => KeyCode::F9,
            CT::F(10) => KeyCode::F10,
            CT::F(11) => KeyCode::F11,
            CT::F(12) => KeyCode::F12,
            CT::Char(' ') | CT::Null => KeyCode::Space,
            CT::Enter => KeyCode::Return,
            CT::Esc => KeyCode::Escape,
            CT::Backspace => KeyCode::Backspace,
            CT::Tab => KeyCode::Tab,
            CT::Left => KeyCode::LeftArrow,
            CT::Right => KeyCode::RightArrow,
            CT::Up => KeyCode::UpArrow,
            CT::Down => KeyCode::DownArrow,
            CT::Char(c) => KeyCode::Other(c as u32),
            _ => KeyCode::Other(0),
        }
    }

    #[cfg(target_arch = "wasm32")]
    use std::cell::RefCell;

    #[cfg(target_arch = "wasm32")]
    thread_local! {
        static KEY_QUEUE: RefCell<std::collections::VecDeque<(KeyCode, KeyState)>> =
            RefCell::new(std::collections::VecDeque::new());
    }

    #[cfg(target_arch = "wasm32")]
    pub fn push_key_event(code: KeyCode, state: KeyState) {
        KEY_QUEUE.with(|q| q.borrow_mut().push_back((code, state)));
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_key_event() -> Option<(KeyCode, KeyState)> {
        KEY_QUEUE.with(|q| q.borrow_mut().pop_front())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn flush_key_buffer() {
        KEY_QUEUE.with(|q| q.borrow_mut().clear());
    }
}
