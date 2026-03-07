# Context (`ctx`)

`ctx` is a shared Lua table injected as a global before `experiment:run()` is called. Structure nodes write to it as the experiment progresses; display and data nodes read from it.

You can also read and write `ctx` freely inside predicates passed to `If` and `Loop`, and inside `ForBlocks` and `ForTrials` callbacks.

---

## Fields

| Field | Type | Set by | Description |
|---|---|---|---|
| `ctx.block` | integer | `ForBlocks` | Current block number, starting at 1. `0` before any block has started. |
| `ctx.trial_index` | integer | `ForTrials` | Global trial counter, incrementing across all blocks. `0` before the first trial. |
| `ctx.trial` | any | `ForTrials` | The current trial value from the list passed to `ForTrials`. `nil` outside a `ForTrials` loop. |
| `ctx.last_response` | table \| nil | `Stimulus` | Response from the most recent `Stimulus` node. `nil` before any stimulus has run, and reset to `nil` at the start of each `ForTrials` iteration. |

### `ctx.last_response` fields

| Field | Type | Description |
|---|---|---|
| `key` | string \| nil | Name of the key pressed (e.g. `"left"`, `"space"`). `nil` on timeout. |
| `rt_ms` | number \| nil | Reaction time in milliseconds from stimulus onset. `nil` on timeout. |
| `timed_out` | boolean | `true` if the response window expired without a keypress. |
| `correct` | boolean \| nil | `true` or `false` if the `Stimulus` node was given a `correct_key`; `nil` otherwise. |

---

## Lifecycle

`Timeline:run()` initialises `ctx` to the following state before executing any nodes:

```lua
ctx.trial_index   = 0
ctx.block         = 0
ctx.trial         = nil
ctx.last_response = nil
```

Each `ForTrials` iteration resets `ctx.last_response` to `nil` before calling the trial callback, so a missed response from one trial does not bleed into the next.

---

## Reading ctx in predicates

`If` and `Loop` predicates are zero-argument functions that may read `ctx` freely:

```lua
-- Show a break screen after every block except the last
If(function() return ctx.block < N_BLOCKS end,
    Instructions({ text = "Take a short break." })
)

-- Loop until the participant responds correctly
Loop(function()
    return ctx.last_response == nil or not ctx.last_response.correct
end, Sequence({
    Stimulus({ stim = probe, keys = {"f","j"}, timeout = 2000 }),
}))
```

---

## Writing custom fields to ctx

You can store your own values on `ctx` for use across nodes. A common pattern is tracking cumulative accuracy:

```lua
ctx.n_correct = 0

ForTrials(trials, function(trial)
    return Sequence({
        Stimulus({ stim = make_stim(trial), keys = {"f","j"}, timeout = 1500,
                   correct_key = trial.correct_key }),
        -- Custom node that increments ctx.n_correct
        { run = function()
            if ctx.last_response and ctx.last_response.correct then
                ctx.n_correct = ctx.n_correct + 1
            end
        end },
        Record({ condition = trial.condition }),
    })
end)
```

Custom fields are never written to the data file automatically, pass them explicitly to `Record` if you want them recorded.