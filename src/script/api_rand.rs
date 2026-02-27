use mlua::prelude::*;
use std::sync::{Arc, Mutex};

/// Using a simple LCG rather than pulling in the `rand` crate,
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed })
    }

    fn from_entropy() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        Self::new(t ^ 0xdeadbeefcafe1234)
    }

    /// xorshift64 — period 2^64-1
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn int(&mut self, lo: i64, hi: i64) -> i64 {
        if lo >= hi {
            return lo;
        }
        let range = (hi - lo + 1) as u64;
        lo + (self.next_u64() % range) as i64
    }

    fn float(&mut self, lo: f64, hi: f64) -> f64 {
        let unit = (self.next_u64() as f64) / (u64::MAX as f64);
        lo + unit * (hi - lo)
    }

    /// Fisher-Yates shuffle of indices 0..n
    fn shuffle_indices(&mut self, n: usize) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            idx.swap(i, j);
        }
        idx
    }
}

pub fn make_rand_table(lua: &Lua, seed: Option<u64>) -> LuaResult<LuaTable> {
    let rng = Arc::new(Mutex::new(match seed {
        Some(s) => Rng::new(s),
        None => Rng::from_entropy(),
    }));

    let t = lua.create_table()?;

    {
        let rng = rng.clone();
        t.set(
            "int",
            lua.create_function(move |_, (lo, hi): (i64, i64)| {
                Ok(rng.lock().expect("rng poisoned").int(lo, hi))
            })?,
        )?;
    }

    {
        let rng = rng.clone();
        t.set(
            "float",
            lua.create_function(move |_, (lo, hi): (f64, f64)| {
                Ok(rng.lock().expect("rng poisoned").float(lo, hi))
            })?,
        )?;
    }

    {
        let rng = rng.clone();
        t.set(
            "shuffle",
            lua.create_function(move |lua_ctx, tbl: LuaTable| {
                let len = tbl.raw_len();
                if len == 0 {
                    return Ok(tbl);
                }

                let indices = rng.lock().expect("rng poisoned").shuffle_indices(len);

                let vals: Vec<LuaValue> = (1..=len)
                    .map(|i| tbl.raw_get::<LuaValue>(i as i64))
                    .collect::<LuaResult<_>>()?;

                let out = lua_ctx.create_table()?;
                for (new_pos, old_pos) in indices.iter().enumerate() {
                    out.raw_set(new_pos as i64 + 1, vals[*old_pos].clone())?;
                }
                Ok(out)
            })?,
        )?;
    }

    {
        let rng = rng.clone();
        t.set(
            "choice",
            lua.create_function(move |_, tbl: LuaTable| {
                let len = tbl.raw_len();
                if len == 0 {
                    return Ok(LuaValue::Nil);
                }
                let i = rng.lock().expect("rng poisoned").int(1, len as i64);
                tbl.raw_get::<LuaValue>(i)
            })?,
        )?;
    }

    {
        let rng = rng.clone();
        t.set(
            "balanced_shuffle",
            lua.create_function(move |lua_ctx, (items, n): (LuaTable, usize)| {
                let item_count = items.raw_len();
                if item_count == 0 || n == 0 {
                    return lua_ctx.create_table().map(Ok)?;
                }

                let mut pool: Vec<LuaValue> = Vec::with_capacity(n);
                for slot in 0..n {
                    let item_idx = (slot % item_count) + 1;
                    pool.push(items.raw_get::<LuaValue>(item_idx as i64)?);
                }

                let indices = rng.lock().expect("rng poisoned").shuffle_indices(n);
                let out = lua_ctx.create_table()?;
                for (new_pos, old_pos) in indices.iter().enumerate() {
                    out.raw_set(new_pos as i64 + 1, pool[*old_pos].clone())?;
                }
                Ok(out)
            })?,
        )?;
    }

    Ok(t)
}
