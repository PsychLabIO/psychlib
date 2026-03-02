use super::HostState;
use crate::clock::Duration;
use crate::io::{
    keyboard::KeyCode,
    response::{ResponseOutcome, ResponseWindow},
};
use mlua::prelude::*;

pub(crate) fn make_trial_table(lua: &Lua, state: &HostState) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    {
        let clock = state.clock.clone();
        let render_handle = state.render_handle.clone();

        t.set(
            "show",
            lua.create_function(move |_, (stim_val, ms): (LuaValue, Option<f64>)| {
                let stim_tbl = match stim_val {
                    LuaValue::Table(tbl) => tbl,
                    other => {
                        return Err(LuaError::runtime(format!(
                            "Trial.show: expected Stimulus, got {}",
                            other.type_name()
                        )));
                    }
                };

                let stim = crate::script::api_stim::lua_to_stim(&stim_tbl)?;

                let flip_instant = {
                    let guard = render_handle
                        .lock()
                        .expect("render_handle mutex poisoned");

                    match guard.as_ref() {
                        Some(handle) => handle
                            .show_and_wait_flip(stim)
                            .map_err(LuaError::external)?,
                        None => {
                            tracing::warn!("Trial.show called but no renderer attached");
                            if let Some(ms) = ms {
                                if ms > 0.0 {
                                    clock.sleep(Duration::from_secs(ms / 1000.0));
                                }
                            }
                            return Ok(());
                        }
                    }
                };

                if let Some(ms) = ms {
                    if ms > 0.0 {
                        let target = flip_instant + Duration::from_secs(ms / 1000.0);
                        clock.sleep_until(target);
                    }
                }

                Ok(())
            })?,
        )?;
    }

    {
        let clock = state.clock.clone();
        let render_handle = state.render_handle.clone();

        t.set(
            "blank",
            lua.create_function(move |_, ms: Option<f64>| {
                let flip_instant = {
                    let guard = render_handle
                        .lock()
                        .expect("render_handle mutex poisoned");

                    match guard.as_ref() {
                        Some(handle) => handle
                            .clear_and_wait_flip()
                            .map_err(LuaError::external)?,
                        None => {
                            tracing::warn!("Trial.blank called but no renderer attached");
                            if let Some(ms) = ms {
                                if ms > 0.0 {
                                    clock.sleep(Duration::from_secs(ms / 1000.0));
                                }
                            }
                            return Ok(());
                        }
                    }
                };

                if let Some(ms) = ms {
                    if ms > 0.0 {
                        let target = flip_instant + Duration::from_secs(ms / 1000.0);
                        clock.sleep_until(target);
                    }
                }

                Ok(())
            })?,
        )?;
    }

    {
        let render_handle = state.render_handle.clone();

        t.set(
            "preload_image",
            lua.create_function(move |_, path: String| {
                let guard = render_handle
                    .lock()
                    .expect("render_handle mutex poisoned");

                let Some(handle) = guard.as_ref() else {
                    return Err(LuaError::runtime(
                        "Trial.preload_image: no renderer attached",
                    ));
                };

                handle
                    .preload_image(&path)
                    .map(|_| ())
                    .map_err(LuaError::external)
            })?,
        )?;
    }

    {
        let clock = state.clock.clone();
        t.set(
            "wait_key",
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
    }

    {
        let trial_index = state.trial_index.clone();
        t.set(
            "next",
            lua.create_function(move |_, ()| {
                let mut idx = trial_index.lock().expect("trial_index mutex poisoned");
                *idx += 1;
                Ok(*idx)
            })?,
        )?;
    }

    {
        let block_index = state.block_index.clone();
        t.set(
            "set_block",
            lua.create_function(move |_, n: usize| {
                *block_index.lock().expect("block_index mutex poisoned") = n;
                Ok(())
            })?,
        )?;
    }

    {
        let trial_index = state.trial_index.clone();
        t.set(
            "trial_index",
            lua.create_function(move |_, ()| {
                Ok(*trial_index.lock().expect("trial_index mutex poisoned"))
            })?,
        )?;
    }

    Ok(t)
}