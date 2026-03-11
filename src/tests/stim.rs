#[allow(unused_imports)]
use crate::renderer::stimulus::{Color, Stimulus};
#[allow(unused_imports)]
use crate::script::api_stim::{lua_to_stim, make_stim_table, parse_color_str, parse_color_value};
use mlua::prelude::*;

#[allow(dead_code)]
fn lua_with_stim() -> Lua {
    let lua = Lua::new();
    let stim = make_stim_table(&lua, 1920.0, 1080.0).unwrap();
    lua.globals().set("Stim", stim).unwrap();
    lua
}

/// Execute a Lua expression and return the result as a LuaTable.
#[allow(dead_code)]
fn eval_table(lua: &Lua, expr: &str) -> LuaTable {
    lua.load(expr)
        .eval::<LuaTable>()
        .unwrap_or_else(|e| panic!("Lua eval failed for `{}`: {}", expr, e))
}

/// Execute Lua, decode the result as a Stimulus.
#[allow(dead_code)]
fn eval_stim(lua: &Lua, expr: &str) -> Stimulus {
    let tbl = eval_table(lua, expr);
    lua_to_stim(&tbl).unwrap_or_else(|e| panic!("lua_to_stim failed: {}", e))
}

/// Execute a Lua snippet that should raise an error; return the error message.
#[allow(dead_code)]
fn eval_err(lua: &Lua, code: &str) -> String {
    lua.load(code).exec().unwrap_err().to_string()
}

#[test]
fn color_str_hex_6() {
    let c = parse_color_str("#FF8800").unwrap();
    assert!((c.r - 1.0).abs() < 0.01);
    assert!((c.g - 0.533).abs() < 0.01);
    assert!((c.b - 0.0).abs() < 0.01);
    assert!((c.a - 1.0).abs() < 0.01);
}

#[test]
fn color_str_hex_8_with_alpha() {
    let c = parse_color_str("#FFFFFF80").unwrap();
    assert!((c.r - 1.0).abs() < 0.01);
    assert!((c.a - 0.502).abs() < 0.01);
}

#[test]
fn color_str_named_white() {
    assert_eq!(parse_color_str("white").unwrap(), Color::WHITE);
}
#[test]
fn color_str_named_black() {
    assert_eq!(parse_color_str("black").unwrap(), Color::BLACK);
}
#[test]
fn color_str_named_red() {
    assert_eq!(parse_color_str("red").unwrap(), Color::RED);
}
#[test]
fn color_str_named_green() {
    assert_eq!(parse_color_str("green").unwrap(), Color::GREEN);
}
#[test]
fn color_str_named_blue() {
    assert_eq!(parse_color_str("blue").unwrap(), Color::BLUE);
}
#[test]
fn color_str_named_gray() {
    assert_eq!(parse_color_str("gray").unwrap(), Color::GRAY);
}
#[test]
fn color_str_named_grey_alias() {
    assert_eq!(parse_color_str("grey").unwrap(), Color::GRAY);
}
#[test]
fn color_str_named_case_insensitive() {
    assert_eq!(parse_color_str("WHITE").unwrap(), Color::WHITE);
    assert_eq!(parse_color_str("White").unwrap(), Color::WHITE);
}
#[test]
fn color_str_named_yellow() {
    assert!(parse_color_str("yellow").is_some());
}
#[test]
fn color_str_named_cyan() {
    assert!(parse_color_str("cyan").is_some());
}
#[test]
fn color_str_named_aqua() {
    assert!(parse_color_str("aqua").is_some());
}
#[test]
fn color_str_named_magenta() {
    assert!(parse_color_str("magenta").is_some());
}
#[test]
fn color_str_named_orange() {
    assert!(parse_color_str("orange").is_some());
}
#[test]
fn color_str_named_purple() {
    assert!(parse_color_str("purple").is_some());
}
#[test]
fn color_str_transparent() {
    let c = parse_color_str("transparent").unwrap();
    assert_eq!(c.a, 0.0);
}
#[test]
fn color_str_unknown_returns_none() {
    assert!(parse_color_str("notacolor").is_none());
    assert!(parse_color_str("").is_none());
    assert!(parse_color_str("#ZZZZZZ").is_none());
}

#[test]
fn color_value_from_hex_string() {
    let lua = Lua::new();
    let val = LuaValue::String(lua.create_string("#FF0000").unwrap());
    let c = parse_color_value(&val).unwrap();
    assert!((c.r - 1.0).abs() < 0.01);
    assert_eq!(c.g, 0.0);
}

#[test]
fn color_value_from_named_string() {
    let lua = Lua::new();
    let val = LuaValue::String(lua.create_string("white").unwrap());
    assert_eq!(parse_color_value(&val).unwrap(), Color::WHITE);
}

#[test]
fn color_value_from_color_table() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.color(\"#FF8800\")").eval().unwrap();
    let val = LuaValue::Table(tbl);
    let c = parse_color_value(&val).unwrap();
    assert!((c.r - 1.0).abs() < 0.01);
}

#[test]
fn color_value_from_rgb_table() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.rgb(255, 128, 0)").eval().unwrap();
    let val = LuaValue::Table(tbl);
    let c = parse_color_value(&val).unwrap();
    assert!((c.r - 1.0).abs() < 0.01);
    assert!((c.g - 0.502).abs() < 0.01);
}

#[test]
fn color_value_nil_returns_none() {
    assert!(parse_color_value(&LuaValue::Nil).is_none());
}

#[test]
fn color_value_plain_table_returns_none() {
    let lua = Lua::new();
    let tbl = lua.create_table().unwrap();
    assert!(parse_color_value(&LuaValue::Table(tbl)).is_none());
}

#[test]
fn stim_color_hex() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.color(\"#FFFFFF\")").eval().unwrap();
    assert_eq!(tbl.get::<String>("__type").unwrap(), "Color");
    assert!((tbl.get::<f32>("r").unwrap() - 1.0).abs() < 0.01);
}

#[test]
fn stim_color_named() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.color(\"red\")").eval().unwrap();
    assert!((tbl.get::<f32>("r").unwrap() - 1.0).abs() < 0.01);
    assert!((tbl.get::<f32>("g").unwrap() - 0.0).abs() < 0.01);
}

#[test]
fn stim_color_unknown_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.color(\"notacolor\")");
    assert!(err.contains("unknown color"), "Error was: {}", err);
}

#[test]
fn stim_rgb_components() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.rgb(255, 0, 128)").eval().unwrap();
    assert!((tbl.get::<f32>("r").unwrap() - 1.0).abs() < 0.01);
    assert!((tbl.get::<f32>("g").unwrap() - 0.0).abs() < 0.01);
    assert!((tbl.get::<f32>("b").unwrap() - 0.502).abs() < 0.01);
    assert!((tbl.get::<f32>("a").unwrap() - 1.0).abs() < 0.01);
}

#[test]
fn stim_rgba_alpha() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.rgba(255, 255, 255, 128)").eval().unwrap();
    assert!((tbl.get::<f32>("a").unwrap() - 0.502).abs() < 0.01);
}

#[test]
fn stim_text_minimal() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hello\")");
    match stim {
        Stimulus::Text { content, opts, pos } => {
            assert_eq!(content, "hello");
            assert!(opts.size >= 0.0);
            assert_eq!(opts.color, Color::WHITE);
            assert!(pos.is_none());
        }
        other => panic!("Expected Text, got {:?}", other),
    }
}

#[test]
fn stim_text_with_size() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"+\", { size = 64 })");
    match stim {
        Stimulus::Text { opts, .. } => assert_eq!(opts.size, 64.0),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_with_hex_color() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { color = \"#FF0000\" })");
    match stim {
        Stimulus::Text { opts, .. } => {
            assert!((opts.color.r - 1.0).abs() < 0.01);
            assert!((opts.color.g - 0.0).abs() < 0.01);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_with_named_color() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { color = \"white\" })");
    match stim {
        Stimulus::Text { opts, .. } => assert_eq!(opts.color, Color::WHITE),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_with_stim_color() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { color = Stim.color(\"blue\") })");
    match stim {
        Stimulus::Text { opts, .. } => assert_eq!(opts.color, Color::BLUE),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_with_align() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { align = \"left\" })");
    match stim {
        Stimulus::Text { opts, .. } => assert_eq!(opts.align, "left"),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_invalid_align_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.text(\"hi\", { align = \"justified\" })");
    assert!(err.contains("align"), "Error was: {}", err);
}

#[test]
fn stim_text_invalid_color_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.text(\"hi\", { color = \"notacolor\" })");
    assert!(err.contains("color"), "Error was: {}", err);
}

#[test]
fn stim_text_zero_size_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.text(\"hi\", { size = 0 })");
    assert!(err.contains("size"), "Error was: {}", err);
}

#[test]
fn stim_text_with_position() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { x = 0.75, y = 0.3 })");
    match stim {
        Stimulus::Text {
            pos: Some((x, y)), ..
        } => {
            assert!((x - 0.75).abs() < 0.001);
            assert!((y - 0.3).abs() < 0.001);
        }
        other => panic!("Expected Text with pos, got {:?}", other),
    }
}

#[test]
fn stim_text_x_only_position() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { x = 0.5 })");
    match stim {
        Stimulus::Text {
            pos: Some((x, y)), ..
        } => {
            assert!((x - 0.5).abs() < 0.001);
            assert!(
                (y - 0.5).abs() < 0.001,
                "y should default to 0.5, got {}",
                y
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_no_opts_no_pos() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\")");
    match stim {
        Stimulus::Text { pos: None, .. } => {}
        Stimulus::Text { pos: Some(p), .. } => panic!("Expected pos=None, got {:?}", p),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_text_with_font() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.text(\"hi\", { font = \"Courier\" })");
    match stim {
        Stimulus::Text { opts, .. } => assert_eq!(opts.font, Some("Courier".into())),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_fixation_defaults() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.fixation()");
    match stim {
        Stimulus::Fixation {
            color,
            arm_len,
            thickness,
        } => {
            assert_eq!(color, Color::WHITE);
            assert!(
                (arm_len - 20.0 / 1080.0).abs() < 0.001,
                "arm_len expected ~{}, got {}",
                20.0 / 1080.0,
                arm_len
            );
            assert!(
                (thickness - 3.0 / 1080.0).abs() < 0.001,
                "thickness expected ~{}, got {}",
                3.0 / 1080.0,
                thickness
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_fixation_custom_color() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.fixation({ color = \"red\" })");
    match stim {
        Stimulus::Fixation { color, .. } => assert_eq!(color, Color::RED),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_fixation_custom_size() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.fixation({ arm_len = 40, thickness = 6 })");
    match stim {
        Stimulus::Fixation {
            arm_len, thickness, ..
        } => {
            assert!(
                (arm_len - 40.0 / 1080.0).abs() < 0.001,
                "arm_len expected ~{}, got {}",
                40.0 / 1080.0,
                arm_len
            );
            assert!(
                (thickness - 6.0 / 1080.0).abs() < 0.001,
                "thickness expected ~{}, got {}",
                6.0 / 1080.0,
                thickness
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_rect_defaults_white() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.rect(0.5, 0.5, 100, 50)");
    match stim {
        Stimulus::Rect { rect, color } => {
            assert_eq!(color, Color::WHITE);
            assert!(
                (rect.cx - 0.0).abs() < 0.001,
                "cx: expected 0.0 (NDC centre), got {}",
                rect.cx
            );
            assert!(
                (rect.hw - 100.0 / 1920.0).abs() < 0.001,
                "hw: expected ~{}, got {}",
                100.0 / 1920.0,
                rect.hw
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_rect_with_hex_color() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.rect(0.5, 0.5, 200, 200, \"#00FF00\")");
    match stim {
        Stimulus::Rect { color, .. } => assert_eq!(color, Color::GREEN),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_rect_invalid_color_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.rect(0.5, 0.5, 100, 100, \"bogus\")");
    assert!(err.contains("color"), "Error: {}", err);
}

#[test]
fn stim_blank_defaults_black() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.blank()");
    match stim {
        Stimulus::Rect { color, rect } => {
            assert_eq!(color, Color::BLACK);
            assert!(
                (rect.hw - 1.0).abs() < 0.001,
                "blank hw should be 1.0, got {}",
                rect.hw
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_blank_custom_color() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.blank(\"gray\")");
    match stim {
        Stimulus::Rect { color, .. } => assert_eq!(color, Color::GRAY),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_blank_with_color_table() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.blank(Stim.rgb(128, 128, 128))");
    match stim {
        Stimulus::Rect { color, .. } => {
            assert!((color.r - 0.502).abs() < 0.01);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_image_minimal() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.image(\"face.png\")");
    match stim {
        Stimulus::Image { path, rect, tint } => {
            assert_eq!(path, "face.png");
            assert!(
                (rect.cx - 0.0).abs() < 0.001,
                "cx: expected 0.0 (NDC centre), got {}",
                rect.cx
            );
            assert!(
                (rect.cy - 0.0).abs() < 0.001,
                "cy: expected 0.0 (NDC centre), got {}",
                rect.cy
            );
            assert!(
                (rect.hw - 200.0 / 1920.0).abs() < 0.001,
                "hw: expected ~{}, got {}",
                200.0 / 1920.0,
                rect.hw
            );
            assert_eq!(tint, Color::WHITE);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_image_with_opts() {
    let lua = lua_with_stim();
    let stim = eval_stim(
        &lua,
        "Stim.image(\"face.png\", { cx=0.75, cy=0.25, hw=300, hh=400 })",
    );
    match stim {
        Stimulus::Image { rect, .. } => {
            assert!(
                (rect.cx - 0.5).abs() < 0.001,
                "cx: expected 0.5 (NDC), got {}",
                rect.cx
            );
            assert!(
                (rect.cy - 0.5).abs() < 0.001,
                "cy: expected 0.5 (NDC), got {}",
                rect.cy
            );
            assert!(
                (rect.hw - 300.0 / 1920.0).abs() < 0.001,
                "hw: expected ~{}, got {}",
                300.0 / 1920.0,
                rect.hw
            );
            assert!(
                (rect.hh - 400.0 / 1080.0).abs() < 0.001,
                "hh: expected ~{}, got {}",
                400.0 / 1080.0,
                rect.hh
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_image_with_tint() {
    let lua = lua_with_stim();
    let stim = eval_stim(&lua, "Stim.image(\"face.png\", { tint = \"red\" })");
    match stim {
        Stimulus::Image { tint, .. } => assert_eq!(tint, Color::RED),
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_composite_two_elements() {
    let lua = lua_with_stim();
    let stim = eval_stim(
        &lua,
        "Stim.composite({ Stim.fixation(), Stim.text(\"go!\") })",
    );
    match stim {
        Stimulus::Composite(parts) => {
            assert_eq!(parts.len(), 2);
            assert!(matches!(parts[0], Stimulus::Fixation { .. }));
            assert!(matches!(parts[1], Stimulus::Text { .. }));
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn stim_composite_empty_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.composite({})");
    assert!(err.contains("empty"), "Error: {}", err);
}

#[test]
fn stim_composite_non_stimulus_errors() {
    let lua = lua_with_stim();
    let err = eval_err(&lua, "Stim.composite({ Stim.fixation(), \"not a stim\" })");
    assert!(err.contains("Stimulus"), "Error: {}", err);
}

#[test]
fn lua_to_stim_rejects_plain_table() {
    let lua = Lua::new();
    let tbl = lua.create_table().unwrap();
    let err = lua_to_stim(&tbl).unwrap_err().to_string();
    assert!(err.contains("Stimulus"), "Error: {}", err);
}

#[test]
fn lua_to_stim_rejects_color_table() {
    let lua = lua_with_stim();
    let tbl: LuaTable = lua.load("Stim.color(\"white\")").eval().unwrap();
    let err = lua_to_stim(&tbl).unwrap_err().to_string();
    assert!(err.contains("Stimulus"), "Error: {}", err);
}

#[test]
fn roundtrip_text() {
    let lua = lua_with_stim();
    let tbl = eval_table(
        &lua,
        "Stim.text(\"hello\", { size = 72, color = \"blue\" })",
    );
    let stim = lua_to_stim(&tbl).unwrap();
    match stim {
        Stimulus::Text { content, opts, .. } => {
            assert_eq!(content, "hello");
            assert_eq!(opts.size, 72.0);
            assert_eq!(opts.color, Color::BLUE);
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn roundtrip_fixation() {
    let lua = lua_with_stim();
    let tbl = eval_table(&lua, "Stim.fixation({ color = \"#00FF00\", arm_len = 40 })");
    let stim = lua_to_stim(&tbl).unwrap();
    match stim {
        Stimulus::Fixation { color, arm_len, .. } => {
            assert_eq!(color, Color::GREEN);
            assert!(
                (arm_len - 40.0 / 1080.0).abs() < 0.001,
                "arm_len: expected ~{}, got {}",
                40.0 / 1080.0,
                arm_len
            );
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn roundtrip_composite() {
    let lua = lua_with_stim();
    let tbl = eval_table(&lua, "Stim.composite({ Stim.blank(), Stim.text(\"x\") })");
    let stim = lua_to_stim(&tbl).unwrap();
    assert!(matches!(stim, Stimulus::Composite(_)));
}
