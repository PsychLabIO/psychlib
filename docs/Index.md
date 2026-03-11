# PsychLib Lua API

Reference for the Lua scripting API exposed to experiment scripts by `psychlib`. All globals are injected automatically, meaning no `require` calls are needed.

---

## Globals

| Global | Description |
|---|---|
| [`Clock`](clock.md) | High-resolution monotonic clock |
| [`Stim`](stim.md) | Stimulus constructors and color helpers |
| [`Rand`](rand.md) | Random number generation and balanced sequence utilities |
| [`Context`](context.md) | Shared runtime context table |
| `psychlib_VERSION` | Version string of the running psychlib build |

---

## Timeline nodes

Experiments are built by composing nodes into a `Timeline`. Nodes are plain Lua tables with a `run()` method. They do nothing until the timeline executes them.

| Node | Category | Description |
|---|---|---|
| [`Timeline()`](nodes.md#timeline) | Structure | Root container. Call `:run()` to execute. |
| [`Sequence({ ... })`](nodes.md#sequence) | Structure | Run child nodes in order. |
| [`ForBlocks(n, fn)`](nodes.md#forblocks) | Structure | Repeat `fn(block)` for `n` blocks. |
| [`ForTrials(list, fn)`](nodes.md#fortrials) | Structure | Iterate a trial list, call `fn(trial)` per trial. |
| [`If(pred, node, else?)`](nodes.md#if) | Flow | Conditionally run a node. |
| [`Loop(pred, node)`](nodes.md#loop) | Flow | Run a node while a predicate holds. |
| [`Instructions({ text, duration? })`](nodes.md#instructions) | Display | Show a text screen, wait for keypress or duration. |
| [`Fixation({ duration })`](nodes.md#fixation) | Display | Show a fixation cross for a fixed duration. |
| [`Stimulus({ stim, keys, timeout, ... })`](nodes.md#stimulus) | Display | Show a stimulus and collect a response. |
| [`Blank({ duration })`](nodes.md#blank) | Display | Silent pause. |
| [`Feedback({ correct_text, incorrect_text, duration })`](nodes.md#feedback) | Display | Show response feedback. |
| [`EndScreen({ text, duration? })`](nodes.md#endscreen) | Display | Final screen at experiment end. |
| [`Record({ ... })`](nodes.md#record) | Data | Write a trial row. |
| [`Save()`](nodes.md#save) | Data | Flush and close the output file. |

---

## Coordinate system

Screen positions use **normalised 0–1 coordinates**, top-left origin, with y increasing downward:

| Point | Meaning |
|---|---|
| `(0.0, 0.0)` | Top-left corner |
| `(0.5, 0.5)` | Centre of screen |
| `(1.0, 1.0)` | Bottom-right corner |

Sizes (`size`, `hw`, `hh`, `arm_len`, `thickness`) are always in **pixels**, independent of screen resolution. Node defaults (fixation arm length, instruction font size, etc.) scale automatically with screen height so they look consistent at any resolution.

---

## Minimal experiment script

```lua
-- Simple task

local N_BLOCKS = 2
local TRIALS_PER_CONDITION = 10
local RESPONSE_KEYS = { "left", "right" }

local function make_trials()
    return Rand.balanced_shuffle({ "left", "right" }, TRIALS_PER_CONDITION * 2)
end

local experiment = Timeline()
experiment:set_format("csv")

experiment:add(Instructions({
    text = "Press LEFT or RIGHT to match the arrow direction.\n\nPress any key to begin.",
}))

experiment:add(ForBlocks(N_BLOCKS, function(block)
    return Sequence({
        ForTrials(Shuffle(make_trials()), function(trial)
            return Sequence({
                Fixation({ duration = 500 }),
                Stimulus({
                    stim        = Stim.text(trial == "left" and "<" or ">",
                                      { size = 64, color = "white", align = "center" }),
                    keys        = RESPONSE_KEYS,
                    timeout     = 2000,
                    correct_key = trial,
                }),
                Blank({ duration = 600 }),
                Record({ direction = trial }),
            })
        end),
        If(function() return ctx.block < N_BLOCKS end,
            Instructions({ text = "Short break.\n\nPress any key to continue." })
        ),
    })
end))

experiment:add(EndScreen({ text = "Done. Thank you!", duration = 2000 }))
experiment:add(Save())

experiment:run()
```