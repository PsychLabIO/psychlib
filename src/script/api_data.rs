use super::HostState;
use crate::data::record::TrialRecord;
use mlua::prelude::*;
use tracing::{debug, info};

pub(crate) fn register(lua: &Lua, state: &HostState) -> LuaResult<()> {
    inject_ctx(lua)?;
    register_write_trial(lua, state)?;
    register_save(lua, state)?;
    register_set_format(lua, state)?;
    Ok(())
}

fn inject_ctx(lua: &Lua) -> LuaResult<()> {
    let ctx = lua.create_table()?;
    lua.globals().set("ctx", ctx)?;
    Ok(())
}

fn register_write_trial(lua: &Lua, state: &HostState) -> LuaResult<()> {
    let state = state.clone();
    lua.globals().set(
        "_psychlib_write_trial",
        lua.create_function(move |_, fields: LuaTable| {
            let row = lua_table_to_fields(&fields)?;
            let record = TrialRecord::builder().merge(row).build();

            debug!(
                "_psychlib_write_trial: trial_index={:?}",
                record
                    .fields
                    .get("trial_index")
                    .and_then(|v| v.as_u64())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "?".to_string())
            );

            let mut store = state.data_store.lock().expect("data_store poisoned");
            if let Some(ref mut mw) = *store {
                mw.write_trial(&record)
                    .map_err(|e| LuaError::runtime(e.to_string()))?;
            }
            Ok(())
        })?,
    )?;
    Ok(())
}

fn register_save(lua: &Lua, state: &HostState) -> LuaResult<()> {
    let data_store = state.data_store.clone();
    lua.globals().set(
        "_psychlib_save",
        lua.create_function(move |_, ()| {
            let mut store = data_store.lock().expect("data_store poisoned");
            if let Some(mw) = store.take() {
                info!("_psychlib_save called");
                mw.close().map_err(|e| LuaError::runtime(e.to_string()))?;
            }
            Ok(())
        })?,
    )?;
    Ok(())
}

fn register_set_format(lua: &Lua, state: &HostState) -> LuaResult<()> {
    let state = state.clone();
    lua.globals().set(
        "_psychlib_set_format",
        lua.create_function(move |_, format: String| {
            info!("_psychlib_set_format: format={}", format);
            state
                .set_format(&format)
                .map_err(|e| LuaError::runtime(e.to_string()))?;
            Ok(())
        })?,
    )?;
    Ok(())
}

fn lua_table_to_fields(
    tbl: &LuaTable,
) -> LuaResult<impl IntoIterator<Item = (String, serde_json::Value)>> {
    const PRIORITY_KEYS: &[&str] = &[
        "trial_index",
        "block",
        "response_key",
        "rt_ms",
        "timed_out",
        "correct",
    ];

    let mut priority: Vec<(String, serde_json::Value)> = Vec::new();
    let mut rest: Vec<(String, serde_json::Value)> = Vec::new();

    for pair in tbl.clone().pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        let key = match &k {
            LuaValue::String(s) => s.to_str()?.to_string(),
            LuaValue::Integer(i) => i.to_string(),
            other => format!("{:?}", other.type_name()),
        };
        let val = lua_value_to_json(v)?;

        if PRIORITY_KEYS.contains(&key.as_str()) {
            priority.push((key, val));
        } else {
            rest.push((key, val));
        }
    }

    priority.sort_by_key(|(k, _)| {
        PRIORITY_KEYS
            .iter()
            .position(|p| p == k)
            .unwrap_or(usize::MAX)
    });

    rest.sort_by(|(a, _), (b, _)| a.cmp(b));

    priority.extend(rest);
    Ok(priority)
}

pub fn lua_value_to_json(v: LuaValue) -> LuaResult<serde_json::Value> {
    Ok(match v {
        LuaValue::Nil => serde_json::Value::Null,
        LuaValue::Boolean(b) => serde_json::Value::Bool(b),
        LuaValue::Integer(i) => serde_json::json!(i),
        LuaValue::Number(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
        ),
        LuaValue::String(s) => serde_json::Value::String(s.to_str()?.to_string()),
        LuaValue::Table(tbl) => {
            let mut map = serde_json::Map::new();
            for pair in tbl.pairs::<LuaValue, LuaValue>() {
                let (k, v) = pair?;
                let key = match k {
                    LuaValue::String(s) => s.to_str()?.to_string(),
                    LuaValue::Integer(i) => i.to_string(),
                    other => format!("{:?}", other.type_name()),
                };
                map.insert(key, lua_value_to_json(v)?);
            }
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::Null,
    })
}