# Data

Trial-level data recording is handled by the `Record` and `Save` [timeline nodes](nodes.md).

---

## Writing data

Place a `Record` node inside your `ForTrials` callback, after the `Stimulus` and `Blank` nodes:

```lua
ForTrials(Shuffle(make_trials()), function(trial)
    return Sequence({
        Fixation({ duration = 500 }),
        Stimulus({
            stim    = make_stim(trial),
            keys    = { "left", "right" },
            timeout = 1500,
            correct_key = trial.direction,
        }),
        Blank({ duration = 800 }),
        Record({
            direction = trial.direction,
            congruent = trial.congruent,
        }),
    })
end)
```

`Record` merges your fields with the automatic runtime columns from `ctx`. You only supply the trial-specific columns.

---

## Automatic columns

The following columns are written automatically by every `Record` node:

| Column | Source | Description |
|---|---|---|
| `trial_index` | `ctx.trial_index` | Global trial counter. |
| `block` | `ctx.block` | Current block number. |
| `response_key` | `ctx.last_response.key` | Key pressed, or absent if timed out. |
| `rt_ms` | `ctx.last_response.rt_ms` | Reaction time in ms, or absent if timed out. |
| `timed_out` | `ctx.last_response.timed_out` | `true` if no response was made. |
| `correct` | `ctx.last_response.correct` | Correctness - only present if `Stimulus` was given a `correct_key`. |

Response columns are omitted entirely (not written as empty strings) when `ctx.last_response` is `nil` - for example, if `Record` is used without a preceding `Stimulus`.

If a key you supply to `Record` has the same name as an automatic column, your value takes precedence.

---

## Saving the file

Add a `Save()` node as the last item in the timeline:

```lua
experiment:add(Save())
experiment:run()
```

`Save()` flushes and closes the writer. After it runs, further `Record` calls have no effect. The file is also closed automatically if the session ends without an explicit `Save()`, but calling it explicitly is good practice.

---

## Output format

The output format is set on the timeline before `run()`:

```lua
local experiment = Timeline()
experiment:set_format("csv")   -- "csv" (default) | "json" | "both"
```

| Format | Output |
|---|---|
| `"csv"` | `<participant>_<script>_<timestamp>.csv` - standard CSV with `#`-prefixed session metadata comments at the top. |
| `"json"` | `<participant>_<script>_<timestamp>.json` - a JSON object with `"session"` and `"trials"` keys. |
| `"both"` | Both files written simultaneously from the same data. |

### CSV format

```
# psychlib_version: 0.2.0
# participant: P001
# script_name: flanker
# started_at: 2026-03-06T04:44:23.560628Z
# seed: entropy
# platform: native-macos
trial_index,block,response_key,rt_ms,timed_out,correct,direction,congruent
1,1,left,312.4,false,true,left,true
2,1,right,489.1,false,false,left,false
...
```

The `#`-prefixed comment lines are ignored by standard CSV parsers. The column order is: automatic runtime columns first (in a fixed order), then your trial-specific columns alphabetically.

### JSON format

```json
{
  "session": {
    "participant": "P001",
    "script_name": "flanker",
    "started_at": "2026-03-06T04:44:23Z",
    ...
  },
  "trials": [
    {
      "trial_index": 1,
      "block": 1,
      "response_key": "left",
      "rt_ms": 312.4,
      "timed_out": false,
      "correct": true,
      "direction": "left",
      "congruent": true
    },
    ...
  ]
}
```