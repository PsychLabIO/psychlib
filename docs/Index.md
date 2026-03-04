# PsychLib Lua API

This is the reference for the Lua scripting API exposed to experiment scripts by `psychlib`. All globals are injected automatically, meaning no `require` calls are needed.

## Globals

| Global | Description |
|---|---|
| [`Clock`](clock.md) | High-resolution monotonic clock |
| [`Trial`](trial.md) | Stimulus display, response collection, trial/block counters |
| [`Data`](data.md) | Trial-level data recording |
| [`Rand`](rand.md) | Random number generation and balanced sequence utilities |
| [`Stim`](stim.md) | Stimulus constructors and color helpers |
| `psychlib_VERSION` | Version string of the running psychlib build |

---

## Coordinate system

Screen positions use **normalised coordinates** where `(0, 0)` is the centre of the display. The value `1.0` reaches the edge of the screen along the shorter axis, so coordinates are consistent across different display resolutions.

---

## Minimal experiment script

```lua
-- One-block, ten-trial reaction-time task

Trial.set_block(1)

local conditions = Rand.balanced_shuffle({"left", "right"}, 10)

for i = 1, #conditions do
    local trial_n = Trial.next()
    local cond = conditions[i]

    Trial.show(Stim.fixation(), 500)
    Trial.blank(Rand.float(400, 600))

    local arrow = cond == "left" and "<-" or "->"
    Trial.show(Stim.text(arrow, { size = 0.15 }))

    local resp = Trial.wait_key({ keys = {"left", "right"}, timeout = 2000 })

    Trial.blank(500)

    Data.record({
        trial = trial_n,
        condition = cond,
        response = resp and resp.key or "timeout",
        rt_ms = resp and resp.rt_ms or nil,
        correct = resp ~= nil and resp.key == cond,
    })
end
```