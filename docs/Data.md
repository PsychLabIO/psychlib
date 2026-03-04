# Data

Writes trial-level data to the session output file.

`Data` is available as a global in every experiment script.

---

## Functions

### `Data.record(fields: table)`

Writes one row to the output file. `fields` is a Lua table of key-value pairs representing the columns you want to record. The following columns are added automatically:

| Column | Description |
|---|---|
| `trial_index` | Current value of the trial counter |
| `block_index` | Current block index |
| `phase` | Always `"Experiment"` for script-initiated records |
| `timestamp_ms` | High-resolution session timestamp (ms) |
| `wall_time` | Wall-clock date/time at the moment of the call |

```lua
Data.record({
    condition  = "congruent",
    stimulus   = "face_01.png",
    response   = resp.key,
    rt_ms      = resp.rt_ms,
    correct    = resp.key == "f",
})
```

Fields may be strings, numbers, booleans, or nested tables (serialised as JSON).

---

### `Data.save()`

Flushes and closes the output file. After this call the writer is consumed and no further `Data.record` calls will have any effect.

You do not normally need to call this. The file is closed automatically when the session ends. Use it only if you need to guarantee the file is written before the script exits, for example when running in a batch pipeline.

```lua
Data.save()
```