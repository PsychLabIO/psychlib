# Timeline Nodes

Experiments in psychlib are built by composing **nodes**, which are plain Lua tables with a single `run()` method. Nodes declare what should happen; the runtime executes them in order when `experiment:run()` is called.

All node constructors are available as globals. No `require` calls are needed.

---

## Structure nodes

### `Timeline()`

The root container. Add nodes with `:add()`, set the output format with `:set_format()`, then call `:run()` to execute the experiment.

```lua
local experiment = Timeline()
experiment:set_format("json")

experiment:add(Instructions({ text = "Welcome." }))
experiment:add(Save())

experiment:run()
```

**Methods:**

| Method | Description |
|---|---|
| `:add(node)` | Append a node to the timeline. |
| `:set_format(format)` | Set the output format before `run()`. See [output formats](#output-formats). |
| `:run()` | Initialise `ctx` and execute all nodes in order. |

#### Output formats

| Format | Files written |
|---|---|
| `"csv"` | `<participant>_<script>_<timestamp>.csv` (default) |
| `"json"` | `<participant>_<script>_<timestamp>.json` |
| `"both"` | Both CSV and JSON simultaneously |

`:set_format()` must be called **before** `run()`. Calling it after the first trial is recorded has no effect.

---

### `Sequence({ node, ... })`

Runs a list of child nodes in order.

```lua
Sequence({
    Fixation({ duration = 500 }),
    Stimulus({ stim = my_stim, keys = {"f","j"}, timeout = 1500 }),
    Blank({ duration = 800 }),
})
```

---

### `ForBlocks(n, fn)`

Calls `fn(block)` for each block from `1` to `n`. `fn` must return a node, which is executed immediately. Sets `ctx.block` for the duration of each iteration.

```lua
ForBlocks(3, function(block)
    return Sequence({
        -- trial sequence here
    })
end)
```

| Parameter | Type | Description |
|---|---|---|
| `n` | integer | Number of blocks. |
| `fn` | `function(block) -> node` | Called lazily for each block. Receives the block number. Must return a node. |

---

### `ForTrials(list, fn)`

Iterates `list` in order, calling `fn(trial)` for each element. `fn` must return a node, which is executed immediately. Sets `ctx.trial` and increments `ctx.trial_index` on each iteration. Clears `ctx.last_response` before each trial.

```lua
ForTrials(Shuffle(make_trials()), function(trial)
    return Sequence({
        Stimulus({ stim = make_stim(trial), keys = {"left","right"}, timeout = 1500 }),
        Record({ condition = trial.condition }),
    })
end)
```

| Parameter | Type | Description |
|---|---|---|
| `list` | table | Flat sequence of trial values. Use `Shuffle()` to randomise. |
| `fn` | `function(trial) -> node` | Called lazily per trial. Receives the current trial value. Must return a node. |

---

## Flow nodes

### `If(predicate, node, else_node?)`

Evaluates `predicate()` and runs `node` if it returns true, or `else_node` if provided and the predicate returns false.

```lua
If(function() return ctx.block < N_BLOCKS end,
    Instructions({ text = "Break. Press any key." })
)
```

```lua
If(function() return ctx.last_response.correct end,
    Feedback({ correct_text = "Correct!", incorrect_text = "", duration = 500 }),
    Feedback({ correct_text = "", incorrect_text = "Wrong.", duration = 500 })
)
```

| Parameter | Type | Description |
|---|---|---|
| `predicate` | `function() -> boolean` | Evaluated at runtime; may read `ctx` freely. |
| `node` | node | Run if predicate is true. |
| `else_node` | node (optional) | Run if predicate is false. |

---

### `Loop(predicate, node)`

Runs `node` repeatedly while `predicate()` returns true. The predicate is evaluated before each iteration (while-style). Use with care, an always-true predicate will loop forever.

```lua
local attempts = 0
Loop(
    function() return attempts < 3 end,
    Sequence({
        Stimulus({ stim = probe, keys = {"space"}, timeout = 2000 }),
    })
)
```

---

## Display nodes

### `Instructions({ text, duration? })`

Renders `text` centred on screen. If `duration` is provided, auto-advances after that many milliseconds. Otherwise waits for any keypress, then shows a 500 ms blank before returning.

```lua
Instructions({
    text = "Press F for faces, H for houses.\n\nPress any key to begin.",
})

Instructions({
    text = "Get ready...",
    duration = 1500,
})
```

| Option | Type | Default | Description |
|---|---|---|---|
| `text` | string | required | Text to display. |
| `duration` | number | nil | If set, auto-advance after this many ms. |

---

### `Fixation({ duration })`

Shows a fixation cross for a fixed duration (ms), then returns.

```lua
Fixation({ duration = 500 })
```

| Option | Type | Default | Description |
|---|---|---|---|
| `duration` | number | required | Display duration in ms. |

---

### `Stimulus({ stim, keys, timeout, correct_key? })`

Shows `stim`, collects a keypress, and writes the result to `ctx.last_response`. The stimulus remains visible until a key is pressed or `timeout` elapses.

```lua
Stimulus({
    stim = Stim.text("<<><<", { size = 0.08, color = "white", align = "center" }),
    keys = { "left", "right" },
    timeout = 1500,
    correct_key = "right",
})
```

| Option | Type | Default | Description |
|---|---|---|---|
| `stim` | Stimulus | required | The stimulus to display. |
| `keys` | string[] | required | Accepted key names. |
| `timeout` | number | required | Max response window in ms. |
| `correct_key` | string | nil | If set, `ctx.last_response.correct` is populated. |

After `Stimulus` runs, `ctx.last_response` is set:

| Field | Type | Description |
|---|---|---|
| `key` | string \| nil | Key name, or `nil` on timeout. |
| `rt_ms` | number \| nil | Reaction time in ms, or `nil` on timeout. |
| `timed_out` | boolean | `true` if no response was made. |
| `correct` | boolean \| nil | `true`/`false` if `correct_key` was set, otherwise `nil`. |

---

### `Blank({ duration })`

Shows a blank screen for `duration` milliseconds.

```lua
Blank({ duration = 800 })
```

| Option | Type | Default | Description |
|---|---|---|---|
| `duration` | number | required | Blank duration in ms. |

---

### `Feedback({ correct_text, incorrect_text, duration })`

Reads `ctx.last_response.correct` and displays the appropriate text for `duration` ms. Must be placed after a `Stimulus` node.

```lua
Feedback({
    correct_text = "Correct!",
    incorrect_text = "Incorrect.",
    duration = 600,
})
```

| Option | Type | Default | Description |
|---|---|---|---|
| `correct_text` | string | required | Shown when `ctx.last_response.correct` is true. |
| `incorrect_text` | string | required | Shown when `ctx.last_response.correct` is false. |
| `duration` | number | required | Display duration in ms. |

> **Note:** `Feedback` requires that `Stimulus` was given a `correct_key`. If `ctx.last_response` is nil or `correct` is nil, a runtime error is raised.

---

### `EndScreen({ text, duration? })`

Displays a final screen at the end of the experiment. Semantically identical to `Instructions` but signals intent in the script. If `duration` is set, auto-advances; otherwise waits for a keypress.

```lua
EndScreen({
    text = "Task complete. Thank you for participating!",
    duration = 3000,
})
```

| Option | Type | Default | Description |
|---|---|---|---|
| `text` | string | required | Text to display. |
| `duration` | number | nil | If set, auto-advance after this many ms. |

---

## Data nodes

### `Record({ ... })`

Writes one row to the output file. The fields you provide are your trial-specific columns. The following columns are merged in automatically from `ctx`:

| Column | Source | Description |
|---|---|---|
| `trial_index` | `ctx.trial_index` | Global trial counter, incremented by `ForTrials`. |
| `block` | `ctx.block` | Current block number, set by `ForBlocks`. |
| `response_key` | `ctx.last_response.key` | Key pressed, or absent if no response was made. |
| `rt_ms` | `ctx.last_response.rt_ms` | Reaction time, or absent if no response was made. |
| `timed_out` | `ctx.last_response.timed_out` | Whether the response window expired. |
| `correct` | `ctx.last_response.correct` | Correctness, or absent if `correct_key` was not set. |

If a key you provide conflicts with an automatic column, your value takes precedence.

```lua
Record({
    direction = trial.direction,
    congruent = trial.congruent,
})
```

---

### `Save()`

Flushes and closes the output file. After `Save()` runs, the writer is consumed and further `Record` calls have no effect.

Normally placed as the last node in the timeline. The file is also closed automatically if the session ends without an explicit `Save()`.

```lua
experiment:add(Save())
```

---

## Utility functions

### `Shuffle(list) -> table`

Returns a shallow-shuffled copy of `list` using the host RNG. The original list is not modified.

```lua
ForTrials(Shuffle(make_trials()), function(trial) ... end)
```

This is a plain function, not a node. It runs immediately when called. Call it inside the `ForBlocks` callback if you want a fresh shuffle each block:

```lua
ForBlocks(N_BLOCKS, function(block)
    return ForTrials(Shuffle(make_trials()), function(trial)
        ...
    end)
end)
```