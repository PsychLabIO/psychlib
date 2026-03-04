# Clock

High-resolution monotonic clock for timing experiment events.

`Clock` is available as a global in every experiment script.

---

## Functions

### `Clock.now_ms() -> number`

Returns the current time as a floating-point number of **milliseconds** since the experiment session started.

```lua
-- e.g. 1523.741
local t = Clock.now_ms()
```

---

### `Clock.now_secs() -> number`

Returns the current time as a floating-point number of **seconds** since the experiment session started.

```lua
 -- e.g. 1.523741
local t = Clock.now_secs()
```

---

### `Clock.sleep(ms: number)`

Blocks the script for (at least) `ms` milliseconds. Values less than or equal to 0 are ignored.

```lua
-- pause for 500 ms
Clock.sleep(500)
```

> **Note:** For stimulus-locked timing, prefer passing a duration directly to `Trial.show` or `Trial.blank`, which measure from the actual display flip rather than from the call site.