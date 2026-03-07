# Clock

High-resolution monotonic clock for timing experiment events.

`Clock` is available as a global in every experiment script.

---

## Functions

### `Clock.now_ms() -> number`

Returns the current time as a floating-point number of **milliseconds** since the experiment session started.

```lua
local t = Clock.now_ms()  -- e.g. 1523.741
```

---

### `Clock.now_secs() -> number`

Returns the current time as a floating-point number of **seconds** since the experiment session started.

```lua
local t = Clock.now_secs()  -- e.g. 1.523741
```

---

### `Clock.sleep(ms: number)`

Blocks the script for (at least) `ms` milliseconds. Values less than or equal to 0 are ignored.

```lua
Clock.sleep(500)  -- pause for 500 ms
```

> **Note:** For stimulus-locked timing, pass a `duration` directly to `Fixation`, `Stimulus`, or `Blank` nodes. They measure from the actual display flip rather than from the call site, giving more accurate inter-stimulus intervals.