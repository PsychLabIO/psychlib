use super::HostState;
use crate::data::record::TrialRecord;
use crate::scheduler::TrialPhase;
use mlua::prelude::*;
use tracing::{debug, info};

pub(crate) fn make_data_table(lua: &Lua, state: &HostState) -> LuaResult<LuaTable> {
    let t = lua.create_table()?;
    {
        let state = state.clone();
        t.set(
            "record",
            lua.create_function(move |_, fields: LuaTable| {
                let custom = lua_table_to_json(&fields)?;

                let now = state.clock.now();
                let wall = state.clock.to_wall_time(now);
                let trial_i = *state.trial_index.lock().expect("trial_index poisoned");
                let block_i = *state.block_index.lock().expect("block_index poisoned");

                let record = TrialRecord::new(
                    trial_i,
                    block_i,
                    TrialPhase::Experiment,
                    None,
                    &[],
                    custom,
                    now,
                    wall,
                );

                debug!("Data.record: trial={} block={}", trial_i, block_i);

                {
                    let mut store = state.data_store.lock().expect("data_store poisoned");
                    if let Some(ref mut writer) = *store {
                        writer
                            .write_trial(&record)
                            .map_err(|e| LuaError::runtime(e.to_string()))?;
                    }
                }

                *state.trial_index.lock().expect("trial_index poisoned") += 1;

                Ok(())
            })?,
        )?;
    }

    {
        let data_store = state.data_store.clone();
        t.set(
            "save",
            lua.create_function(move |_, ()| {
                let mut store = data_store.lock().expect("data_store poisoned");
                if let Some(writer) = store.take() {
                    info!("Data.save() called from script");
                    Box::new(writer)
                        .close()
                        .map_err(|e| LuaError::runtime(e.to_string()))?;
                }
                Ok(())
            })?,
        )?;
    }

    {
        let trial_index = state.trial_index.clone();
        t.set(
            "trial_index",
            lua.create_function(move |_, ()| {
                Ok(*trial_index.lock().expect("trial_index poisoned"))
            })?,
        )?;
    }

    Ok(t)
}

/// Recursively convert a Luau table into a `serde_json::Value`.
pub fn lua_table_to_json(tbl: &LuaTable) -> LuaResult<serde_json::Value> {
    use serde_json::{Map, Value};

    let mut is_array = true;
    let mut max_int = 0usize;
    let mut has_str = false;

    for pair in tbl.clone().pairs::<LuaValue, LuaValue>() {
        let (k, _) = pair?;
        match k {
            LuaValue::Integer(i) if i >= 1 => {
                max_int = max_int.max(i as usize);
            }
            LuaValue::String(_) => {
                has_str = true;
                is_array = false;
            }
            _ => {
                is_array = false;
            }
        }
    }

    if is_array && !has_str && max_int > 0 {
        let mut arr = vec![Value::Null; max_int];
        for pair in tbl.clone().pairs::<LuaValue, LuaValue>() {
            let (k, v) = pair?;
            if let LuaValue::Integer(i) = k {
                arr[(i as usize) - 1] = lua_value_to_json(v)?;
            }
        }
        Ok(Value::Array(arr))
    } else {
        let mut map = Map::new();
        for pair in tbl.clone().pairs::<LuaValue, LuaValue>() {
            let (k, v) = pair?;
            let key = match k {
                LuaValue::String(s) => s.to_str()?.to_string(),
                LuaValue::Integer(i) => i.to_string(),
                other => format!("{:?}", other.type_name()),
            };
            map.insert(key, lua_value_to_json(v)?);
        }
        Ok(Value::Object(map))
    }
}

fn lua_value_to_json(v: LuaValue) -> LuaResult<serde_json::Value> {
    Ok(match v {
        LuaValue::Nil => serde_json::Value::Null,
        LuaValue::Boolean(b) => serde_json::Value::Bool(b),
        LuaValue::Integer(i) => serde_json::json!(i),
        LuaValue::Number(f) => serde_json::Value::Number(
            serde_json::Number::from_f64(f).unwrap_or(serde_json::Number::from(0)),
        ),
        LuaValue::String(s) => serde_json::Value::String(s.to_str()?.to_string()),
        LuaValue::Table(tbl) => lua_table_to_json(&tbl)?,
        _ => serde_json::Value::Null,
    })
}