use crate::renderer::stimulus::{Color, Rect, Stimulus, TextOptions};
use mlua::prelude::*;

pub fn make_stim_table(lua: &Lua) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    t.set(
        "text",
        lua.create_function(|lua_ctx, (content, opts): (String, Option<LuaTable>)| {
            let text_opts = parse_text_opts(opts.as_ref())?;
            let pos = parse_pos(opts.as_ref());
            stim_to_lua(
                lua_ctx,
                Stimulus::Text {
                    content,
                    opts: text_opts,
                    pos,
                },
            )
        })?,
    )?;

    t.set(
        "fixation",
        lua.create_function(|lua_ctx, opts: Option<LuaTable>| {
            let color = opt_color(opts.as_ref(), "color", Color::WHITE)?;
            let arm_len = opt_f32(opts.as_ref(), "arm_len", 0.03);
            let thickness = opt_f32(opts.as_ref(), "thickness", 0.005);
            stim_to_lua(
                lua_ctx,
                Stimulus::Fixation {
                    color,
                    arm_len,
                    thickness,
                },
            )
        })?,
    )?;

    t.set(
        "rect",
        lua.create_function(
            |lua_ctx, (cx, cy, hw, hh, color_v): (f32, f32, f32, f32, Option<LuaValue>)| {
                let color = color_v
                    .as_ref()
                    .filter(|v| !matches!(v, LuaValue::Nil))
                    .map(|v| parse_color_value(v).ok_or_else(|| color_err("Stim.rect")))
                    .transpose()?
                    .unwrap_or(Color::WHITE);
                stim_to_lua(
                    lua_ctx,
                    Stimulus::Rect {
                        rect: Rect::new(cx, cy, hw, hh),
                        color,
                    },
                )
            },
        )?,
    )?;

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

    t.set(
        "image",
        lua.create_function(|lua_ctx, (path, opts): (String, Option<LuaTable>)| {
            let cx = opt_f32(opts.as_ref(), "cx", 0.0);
            let cy = opt_f32(opts.as_ref(), "cy", 0.0);
            let hw = opt_f32(opts.as_ref(), "hw", 0.5);
            let hh = opt_f32(opts.as_ref(), "hh", 0.5);
            let tint = opt_color(opts.as_ref(), "tint", Color::WHITE)?;
            stim_to_lua(
                lua_ctx,
                Stimulus::Image {
                    path,
                    rect: Rect::new(cx, cy, hw, hh),
                    tint,
                },
            )
        })?,
    )?;

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

    Ok(t)
}

/// Encode a `Stimulus` as `{ __type = "Stimulus", __json = "..." }`.
fn stim_to_lua(lua: &Lua, stim: Stimulus) -> LuaResult<LuaTable> {
    let json = serde_json::to_string(&stim)
        .map_err(|e| LuaError::runtime(format!("stimulus serialization: {e}")))?;
    let tbl = lua.create_table()?;
    tbl.set("__type", "Stimulus")?;
    tbl.set("__json", json)?;
    Ok(tbl)
}

/// Decode a `Stimulus` from a table produced by `stim_to_lua`.
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

/// Decode a `Color` from any valid Lua color representation.
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
        "magenta" | "fuchsia" => Color::new(1.0, 0.0, 1.0, 1.0),
        "orange" => Color::new(1.0, 0.647, 0.0, 1.0),
        "purple" => Color::new(0.502, 0.0, 0.502, 1.0),
        "transparent" => Color::new(0.0, 0.0, 0.0, 0.0),
        _ => return None,
    })
}

fn parse_text_opts(opts: Option<&LuaTable>) -> LuaResult<TextOptions> {
    let mut out = TextOptions::default();
    let Some(o) = opts else { return Ok(out) };

    if let Some(size) = o.get::<Option<f32>>("size")? {
        if size <= 0.0 {
            return Err(LuaError::runtime("Stim.text: size must be > 0"));
        }
        out.size = size;
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

fn parse_pos(opts: Option<&LuaTable>) -> Option<(f32, f32)> {
    let o = opts?;
    let x: Option<f32> = o.get::<Option<f32>>("x").ok().flatten();
    let y: Option<f32> = o.get::<Option<f32>>("y").ok().flatten();
    match (x, y) {
        (None, None) => None,
        (Some(x), Some(y)) => Some((x, y)),
        (Some(x), None) => Some((x, 0.0)),
        (None, Some(y)) => Some((0.0, y)),
    }
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
