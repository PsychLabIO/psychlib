use super::HostState;
use crate::clock::Duration;
use crate::clock::Instant;
use crate::io::{
    keyboard::KeyCode,
    response::{ResponseOutcome, ResponseWindow},
};
use mlua::prelude::*;

pub(crate) fn register(lua: &Lua, state: &HostState) -> LuaResult<()> {
    register_show(lua, state)?;
    register_blank(lua, state)?;
    register_wait_key(lua, state)?;
    Ok(())
}

fn register_show(lua: &Lua, state: &HostState) -> LuaResult<()> {
    let clock = state.clock.clone();
    let render_handle = state.render_handle.clone();

    lua.globals().set(
        "_psychlib_show",
        lua.create_function(move |lua_ctx, (stim_val, ms): (LuaValue, Option<f64>)| {
            let stim_tbl = match stim_val {
                LuaValue::Table(tbl) => tbl,
                other => {
                    return Err(LuaError::runtime(format!(
                        "_psychlib_show: expected Stimulus, got {}",
                        other.type_name()
                    )));
                }
            };

            let stim = crate::script::api_stim::lua_to_stim(&stim_tbl)?;

            let flip_instant = {
                let guard = render_handle.lock().expect("render_handle mutex poisoned");

                match guard.as_ref() {
                    Some(handle) => handle
                        .show_and_wait_flip(stim)
                        .map_err(LuaError::external)?,
                    None => {
                        tracing::warn!("_psychlib_show called but no renderer attached");
                        if let Some(ms) = ms {
                            if ms > 0.0 {
                                clock.sleep(Duration::from_secs(ms / 1000.0));
                            }
                        }
                        return Ok(LuaValue::Nil);
                    }
                }
            };

            if let Some(ms) = ms {
                if ms > 0.0 {
                    let target = flip_instant + Duration::from_secs(ms / 1000.0);
                    clock.sleep_until(target);
                }
            }

            Ok(LuaValue::Table(instant_to_lua(&lua_ctx, flip_instant)?))
        })?,
    )?;
    Ok(())
}

fn register_blank(lua: &Lua, state: &HostState) -> LuaResult<()> {
    let clock = state.clock.clone();
    let render_handle = state.render_handle.clone();

    lua.globals().set(
        "_psychlib_blank",
        lua.create_function(move |lua_ctx, ms: Option<f64>| {
            let flip_instant = {
                let guard = render_handle.lock().expect("render_handle mutex poisoned");

                match guard.as_ref() {
                    Some(handle) => handle.clear_and_wait_flip().map_err(LuaError::external)?,
                    None => {
                        tracing::warn!("_psychlib_blank called but no renderer attached");
                        if let Some(ms) = ms {
                            if ms > 0.0 {
                                clock.sleep(Duration::from_secs(ms / 1000.0));
                            }
                        }
                        return Ok(LuaValue::Nil);
                    }
                }
            };

            if let Some(ms) = ms {
                if ms > 0.0 {
                    let target = flip_instant + Duration::from_secs(ms / 1000.0);
                    clock.sleep_until(target);
                }
            }

            Ok(LuaValue::Table(instant_to_lua(&lua_ctx, flip_instant)?))
        })?,
    )?;
    Ok(())
}

fn register_wait_key(lua: &Lua, state: &HostState) -> LuaResult<()> {
    let clock = state.clock.clone();

    lua.globals().set(
        "_psychlib_wait_key",
        lua.create_function(move |lua_ctx, opts: Option<LuaTable>| {
            let (accepted_keys, timeout_ms): (Vec<KeyCode>, Option<f64>) = match opts {
                None => (vec![], None),
                Some(ref tbl) => {
                    let keys: Vec<KeyCode> = match tbl.get::<LuaValue>("keys")? {
                        LuaValue::Table(keys_tbl) => {
                            let mut out = Vec::new();
                            for pair in keys_tbl.pairs::<LuaValue, LuaString>() {
                                let (_, s) = pair?;
                                if let Some(k) = KeyCode::from_name(&&s.to_str()?) {
                                    out.push(k);
                                }
                            }
                            out
                        }
                        _ => vec![],
                    };
                    let timeout: Option<f64> = tbl.get("timeout")?;
                    (keys, timeout)
                }
            };

            let mut window = ResponseWindow::new(&clock).accept_keys(&accepted_keys);

            if let Some(ms) = timeout_ms {
                window = window.timeout(Duration::from_secs(ms / 1000.0));
            }

            window.arm();
            let outcome = window.wait();

            match outcome {
                ResponseOutcome::Timeout => Ok(LuaValue::Nil),
                ResponseOutcome::Response(r) => {
                    let tbl = lua_ctx.create_table()?;
                    tbl.set("key", r.key.as_name())?;
                    tbl.set("rt_ms", r.rt_ms())?;
                    tbl.set("rt_ns", r.rt.as_nanos())?;
                    Ok(LuaValue::Table(tbl))
                }
            }
        })?,
    )?;
    Ok(())
}

fn instant_to_lua(lua: &Lua, instant: Instant) -> LuaResult<LuaTable> {
    let tbl = lua.create_table()?;
    tbl.set("ns", instant.as_nanos())?;
    tbl.set("ms", instant.as_millis())?;
    tbl.set("secs", instant.as_secs_f64())?;
    Ok(tbl)
}
