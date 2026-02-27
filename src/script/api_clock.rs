use super::HostState;
use mlua::prelude::*;

pub(crate) fn make_clock_table(lua: &Lua, state: &HostState) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    {
        let clock = state.clock.clone();
        t.set(
            "now_ms",
            lua.create_function(move |_, ()| Ok(clock.now().as_secs_f64() * 1000.0))?,
        )?;
    }

    {
        let clock = state.clock.clone();
        t.set(
            "now_secs",
            lua.create_function(move |_, ()| Ok(clock.now().as_secs_f64()))?,
        )?;
    }

    {
        let clock = state.clock.clone();
        t.set(
            "sleep",
            lua.create_function(move |_, ms: f64| {
                if ms > 0.0 {
                    clock.sleep(crate::clock::Duration::from_secs(ms / 1000.0));
                }
                Ok(())
            })?,
        )?;
    }

    Ok(t)
}
