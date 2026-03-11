use crate::renderer::stimulus::{Color, Rect, Stimulus, TextOptions};
use mlua::prelude::*;

/// Convert a 0–1 top-left x position to NDC (-1 to +1).
#[inline]
fn x_to_ndc(x: f32) -> f32 {
    x * 2.0 - 1.0
}

/// Convert a 0–1 top-left y position to NDC (-1 to +1, y-up).
#[inline]
fn y_to_ndc(y: f32) -> f32 {
    -(y * 2.0 - 1.0)
}

/// Convert a pixel half-dimension to NDC space along the x axis.
#[inline]
fn px_hw_to_ndc(px: f32, screen_w: f32) -> f32 {
    px / screen_w
}

/// Convert a pixel half-dimension to NDC space along the y axis.
#[inline]
fn px_hh_to_ndc(px: f32, screen_h: f32) -> f32 {
    px / screen_h
}

/// Convert a pixel size (font size, arm length, etc.) to NDC height units.
#[inline]
fn px_to_ndc_h(px: f32, screen_h: f32) -> f32 {
    px / screen_h
}

/// Build a `Rect` from script-space centre position (0–1) and pixel
/// half-dimensions, converting to NDC for the renderer.
fn rect_from_script(
    cx: f32,
    cy: f32,
    hw_px: f32,
    hh_px: f32,
    screen_w: f32,
    screen_h: f32,
) -> Rect {
    Rect::new(
        x_to_ndc(cx),
        y_to_ndc(cy),
        px_hw_to_ndc(hw_px, screen_w),
        px_hh_to_ndc(hh_px, screen_h),
    )
}

/// Fallback screen size for unit-test
#[allow(dead_code)]
const DEFAULT_SCREEN_W: f32 = 1920.0;
#[allow(dead_code)]
const DEFAULT_SCREEN_H: f32 = 1080.0;

pub fn make_stim_table(lua: &Lua, screen_w: f32, screen_h: f32) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    {
        let sh = screen_h;
        t.set(
            "text",
            lua.create_function(
                move |lua_ctx, (content, opts): (String, Option<LuaTable>)| {
                    let text_opts = parse_text_opts(opts.as_ref(), sh)?;

                    let has_x = opts
                        .as_ref()
                        .and_then(|o| o.get::<Option<f32>>("x").ok().flatten())
                        .is_some();
                    let has_y = opts
                        .as_ref()
                        .and_then(|o| o.get::<Option<f32>>("y").ok().flatten())
                        .is_some();
                    let pos = if has_x || has_y {
                        let x = opt_f32(opts.as_ref(), "x", 0.5);
                        let y = opt_f32(opts.as_ref(), "y", 0.5);
                        Some((x, y))
                    } else {
                        None
                    };
                    stim_to_lua(
                        lua_ctx,
                        Stimulus::Text {
                            content,
                            opts: text_opts,
                            pos,
                        },
                    )
                },
            )?,
        )?;
    }

    {
        let sh = screen_h;
        t.set(
            "fixation",
            lua.create_function(move |lua_ctx, opts: Option<LuaTable>| {
                let color = opt_color(opts.as_ref(), "color", Color::WHITE)?;
                let arm_len_px = opt_f32(opts.as_ref(), "arm_len", 20.0);
                let thickness_px = opt_f32(opts.as_ref(), "thickness", 3.0);
                stim_to_lua(
                    lua_ctx,
                    Stimulus::Fixation {
                        color,
                        arm_len: px_to_ndc_h(arm_len_px, sh),
                        thickness: px_to_ndc_h(thickness_px, sh),
                    },
                )
            })?,
        )?;
    }

    {
        let sw = screen_w;
        let sh = screen_h;
        t.set(
            "rect",
            lua.create_function(
                move |lua_ctx, (cx, cy, hw_px, hh_px, color_v): (f32, f32, f32, f32, Option<LuaValue>)| {
                    let color = color_v
                        .as_ref()
                        .filter(|v| !matches!(v, LuaValue::Nil))
                        .map(|v| parse_color_value(v).ok_or_else(|| color_err("Stim.rect")))
                        .transpose()?
                        .unwrap_or(Color::WHITE);
                    stim_to_lua(
                        lua_ctx,
                        Stimulus::Rect {
                            rect: rect_from_script(cx, cy, hw_px, hh_px, sw, sh),
                            color,
                        },
                    )
                },
            )?,
        )?;
    }

    t.set(
        "blank",
        lua.create_function(|lua_ctx, color_v: Option<LuaValue>| {
            let color = color_v
                .as_ref()
                .filter(|v| !matches!(v, LuaValue::Nil))
                .map(|v| parse_color_value(v).ok_or_else(|| color_err("Stim.blank")))
                .transpose()?
                .unwrap_or(Color::BLACK);
            stim_to_lua(lua_ctx, Stimulus::blank(color))
        })?,
    )?;

    {
        let sw = screen_w;
        let sh = screen_h;
        t.set(
            "image",
            lua.create_function(move |lua_ctx, (path, opts): (String, Option<LuaTable>)| {
                let cx = opt_f32(opts.as_ref(), "cx", 0.5);
                let cy = opt_f32(opts.as_ref(), "cy", 0.5);
                let hw_px = opt_f32(opts.as_ref(), "hw", 200.0);
                let hh_px = opt_f32(opts.as_ref(), "hh", 200.0);
                let tint = opt_color(opts.as_ref(), "tint", Color::WHITE)?;
                stim_to_lua(
                    lua_ctx,
                    Stimulus::Image {
                        path,
                        rect: rect_from_script(cx, cy, hw_px, hh_px, sw, sh),
                        tint,
                    },
                )
            })?,
        )?;
    }

    t.set(
        "composite",
        lua.create_function(|lua_ctx, parts: LuaTable| {
            let mut stimuli = Vec::new();
            for val in parts.sequence_values::<LuaValue>() {
                match val? {
                    LuaValue::Table(tbl) => stimuli.push(lua_to_stim(&tbl)?),
                    other => {
                        return Err(LuaError::runtime(format!(
                            "Stim.composite: expected Stimulus, got {}",
                            other.type_name()
                        )));
                    }
                }
            }
            if stimuli.is_empty() {
                return Err(LuaError::runtime("Stim.composite: parts table is empty"));
            }
            stim_to_lua(lua_ctx, Stimulus::Composite(stimuli))
        })?,
    )?;

    t.set(
        "color",
        lua.create_function(|lua_ctx, spec: String| {
            let color = parse_color_str(&spec).ok_or_else(|| {
                LuaError::runtime(format!(
                    "Stim.color: unknown color {:?} \
                     (use \"#RRGGBB\", \"#RRGGBBAA\", or a named color such as \"white\")",
                    spec
                ))
            })?;
            color_to_lua(lua_ctx, color)
        })?,
    )?;

    t.set(
        "rgb",
        lua.create_function(|lua_ctx, (r, g, b): (f32, f32, f32)| {
            color_to_lua(lua_ctx, Color::new(r / 255.0, g / 255.0, b / 255.0, 1.0))
        })?,
    )?;

    t.set(
        "rgba",
        lua.create_function(|lua_ctx, (r, g, b, a): (f32, f32, f32, f32)| {
            color_to_lua(
                lua_ctx,
                Color::new(r / 255.0, g / 255.0, b / 255.0, a / 255.0),
            )
        })?,
    )?;

    t.set("preload", lua.create_function(|_, _path: String| Ok(()))?)?;

    Ok(t)
}

fn stim_to_lua(lua: &Lua, stim: Stimulus) -> LuaResult<LuaTable> {
    let json = serde_json::to_string(&stim)
        .map_err(|e| LuaError::runtime(format!("stimulus serialization: {e}")))?;
    let tbl = lua.create_table()?;
    tbl.set("__type", "Stimulus")?;
    tbl.set("__json", json)?;
    Ok(tbl)
}

pub fn lua_to_stim(tbl: &LuaTable) -> LuaResult<Stimulus> {
    let ty: Option<String> = tbl.get("__type").ok();
    if ty.as_deref() != Some("Stimulus") {
        return Err(LuaError::runtime(
            "expected a Stimulus (created by Stim.*) but got a plain table",
        ));
    }
    let json: String = tbl
        .get("__json")
        .map_err(|_| LuaError::runtime("Stimulus table missing __json"))?;
    serde_json::from_str(&json)
        .map_err(|e| LuaError::runtime(format!("stimulus deserialization: {e}")))
}

pub use lua_to_stim as lua_stim_to_rust;

pub fn color_to_lua(lua: &Lua, c: Color) -> LuaResult<LuaTable> {
    let tbl = lua.create_table()?;
    tbl.set("__type", "Color")?;
    tbl.set("r", c.r)?;
    tbl.set("g", c.g)?;
    tbl.set("b", c.b)?;
    tbl.set("a", c.a)?;
    Ok(tbl)
}

pub fn parse_color_value(val: &LuaValue) -> Option<Color> {
    match val {
        LuaValue::Table(tbl) => {
            if tbl.get::<String>("__type").ok()?.as_str() != "Color" {
                return None;
            }
            Some(Color::new(
                tbl.get("r").ok()?,
                tbl.get("g").ok()?,
                tbl.get("b").ok()?,
                tbl.get("a").unwrap_or(1.0),
            ))
        }
        LuaValue::String(s) => parse_color_str(&s.to_str().ok()?),
        _ => None,
    }
}

pub fn parse_color_str(s: &str) -> Option<Color> {
    if s.starts_with('#') {
        return Color::from_hex(s);
    }
    Some(match s.to_lowercase().as_str() {
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "red" => Color::RED,
        "green" => Color::GREEN,
        "blue" => Color::BLUE,
        "gray" | "grey" => Color::GRAY,
        "yellow" => Color::new(1.0, 1.0, 0.0, 1.0),
        "cyan" | "aqua" => Color::new(0.0, 1.0, 1.0, 1.0),
        "magenta" => Color::new(1.0, 0.0, 1.0, 1.0),
        "orange" => Color::new(1.0, 0.647, 0.0, 1.0),
        "purple" => Color::new(0.502, 0.0, 0.502, 1.0),
        "transparent" => Color::new(0.0, 0.0, 0.0, 0.0),
        _ => return None,
    })
}

fn parse_text_opts(opts: Option<&LuaTable>, screen_h: f32) -> LuaResult<TextOptions> {
    let _ = screen_h; // fix later
    let mut out = TextOptions::default();
    let Some(o) = opts else { return Ok(out) };

    if let Some(size_px) = o.get::<Option<f32>>("size")? {
        if size_px <= 0.0 {
            return Err(LuaError::runtime("Stim.text: size must be > 0"));
        }
        out.size = size_px;
    }

    let color_val: LuaValue = o.get("color")?;
    if !matches!(color_val, LuaValue::Nil) {
        out.color = parse_color_value(&color_val).ok_or_else(|| {
            LuaError::runtime(
                "Stim.text: invalid color \
                 (use \"#RRGGBB\", a named color, or Stim.color(...))",
            )
        })?;
    }

    if let Some(align) = o.get::<Option<String>>("align")? {
        match align.as_str() {
            "left" | "center" | "right" => out.align = align,
            other => {
                return Err(LuaError::runtime(format!(
                    "Stim.text: align must be \"left\", \"center\", or \"right\", got {:?}",
                    other
                )));
            }
        }
    }

    if let Some(font) = o.get::<Option<String>>("font")? {
        out.font = Some(font);
    }

    Ok(out)
}

fn opt_color(opts: Option<&LuaTable>, key: &str, default: Color) -> LuaResult<Color> {
    let Some(o) = opts else { return Ok(default) };
    let val: LuaValue = o.get(key)?;
    if matches!(val, LuaValue::Nil) {
        return Ok(default);
    }
    parse_color_value(&val).ok_or_else(|| color_err(key))
}

fn opt_f32(opts: Option<&LuaTable>, key: &str, default: f32) -> f32 {
    opts.and_then(|o| o.get::<Option<f32>>(key).ok().flatten())
        .unwrap_or(default)
}

fn color_err(ctx: &str) -> LuaError {
    LuaError::runtime(format!(
        "{ctx}: invalid color (use \"#RRGGBB\", a named color, or Stim.color(...))"
    ))
}
