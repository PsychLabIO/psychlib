use serde::{Deserialize, Serialize};

/// An RGBA colour, components in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const GRAY: Self = Self {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a CSS hex string: `"#RRGGBB"` or `"#RRGGBBAA"`.
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        let parse_byte = |i: usize| -> Option<f32> {
            u8::from_str_radix(&s[i..i + 2], 16)
                .ok()
                .map(|b| b as f32 / 255.0)
        };
        match s.len() {
            6 => Some(Self {
                r: parse_byte(0)?,
                g: parse_byte(2)?,
                b: parse_byte(4)?,
                a: 1.0,
            }),
            8 => Some(Self {
                r: parse_byte(0)?,
                g: parse_byte(2)?,
                b: parse_byte(4)?,
                a: parse_byte(6)?,
            }),
            _ => None,
        }
    }

    pub fn to_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextOptions {
    pub size: f32,
    pub color: Color,
    pub align: String,
    pub font: Option<String>,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            size: 48.0,
            color: Color::WHITE,
            align: "center".to_string(),
            font: None,
        }
    }
}

/// A rectangle in normalised screen coordinates [-1, 1].
/// (0, 0) is screen centre.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
}

impl Rect {
    pub const FULLSCREEN: Self = Self {
        cx: 0.0,
        cy: 0.0,
        hw: 1.0,
        hh: 1.0,
    };

    pub fn new(cx: f32, cy: f32, hw: f32, hh: f32) -> Self {
        Self { cx, cy, hw, hh }
    }

    pub fn from_pixels(
        cx_px: f32,
        cy_px: f32,
        w_px: f32,
        h_px: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> Self {
        Self {
            cx: (cx_px / screen_w) * 2.0 - 1.0,
            cy: 1.0 - (cy_px / screen_h) * 2.0,
            hw: w_px / screen_w,
            hh: h_px / screen_h,
        }
    }
}

/// Everything a script can ask the renderer to display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stimulus {
    /// Solid-colour rectangle
    Rect { rect: Rect, color: Color },

    /// UTF-8 text string
    Text {
        content: String,
        opts: TextOptions,
        /// Position override (None = screen centre).
        pos: Option<(f32, f32)>,
    },

    /// Rendered as two overlapping rects, not text, to avoid font loading.
    Fixation {
        color: Color,
        arm_len: f32,
        thickness: f32,
    },

    /// Image loaded from a path.
    Image {
        path: String,
        rect: Rect,
        /// Tint colour (default: white = no tint).
        tint: Color,
    },

    Composite(Vec<Stimulus>),
}

impl Stimulus {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            opts: TextOptions::default(),
            pos: None,
        }
    }

    pub fn text_with(content: impl Into<String>, opts: TextOptions) -> Self {
        Self::Text {
            content: content.into(),
            opts,
            pos: None,
        }
    }

    pub fn fixation() -> Self {
        Self::Fixation {
            color: Color::WHITE,
            arm_len: 0.03,
            thickness: 0.005,
        }
    }

    pub fn blank(color: Color) -> Self {
        Self::Rect {
            rect: Rect::FULLSCREEN,
            color,
        }
    }
}
