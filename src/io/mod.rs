pub mod keyboard;
pub mod response;

pub use keyboard::{KeyCode, KeyState, PhysicalKey};
pub use response::{Response, ResponseOutcome, ResponseWindow};

use crate::clock::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub timestamp: Instant,
    pub kind: InputKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum InputKind {
    Key {
        code: KeyCode,
        state: KeyState,
    },

    MouseButton {
        button: MouseButton,
        state: KeyState,
        x: f32,
        y: f32,
    },

    Serial {
        port: u8,
        value: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}
