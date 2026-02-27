use super::HostState;
use crate::clock::Duration;
use crate::io::{
    keyboard::KeyCode,
    response::{ResponseOutcome, ResponseWindow},
};
use mlua::prelude::*;
use tracing::debug;

pub(crate) fn make_trial_table(lua: &Lua, state: &HostState) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;

    {
        let clock = state.clock.clone();
        t.set(
            "blank",
            lua.create_function(move |_, ms: Option<f64>| {
                match ms {
                    Some(ms) if ms > 0.0 => {
                        clock.sleep(Duration::from_secs(ms / 1000.0));
                    }
                    Some(_) => {}
                    None => {
                        let mut w = ResponseWindow::new(&clock);
                        w.arm();
                        w.wait();
                    }
                }
                Ok(())
            })?,
        )?;
    }

    {
        let clock = state.clock.clone();
        t.set(
            "show",
            lua.create_function(move |_, (stim, ms): (LuaValue, Option<f64>)| {
                debug!(
                    "Trial.show called (renderer stub): stim={:?} ms={:?}",
                    stim.type_name(),
                    ms
                );
                match ms {
                    Some(ms) if ms > 0.0 => {
                        clock.sleep(Duration::from_secs(ms / 1000.0));
                    }
                    None => {}
                    _ => {}
                }
                Ok(())
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
