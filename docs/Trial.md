# Trial

Controls stimulus presentation, response collection, and trial/block bookkeeping.

`Trial` is available as a global in every experiment script.

---

## Types

### `Instant`

Returned by display functions. Represents the moment the display flip occurred.

| Field | Type | Description |
|---|---|---|
| `ns` | integer | Nanoseconds since session start |
| `ms` | number | Milliseconds since session start |
| `secs` | number | Seconds since session start |

### `KeyResponse`

Returned by `Trial.wait_key` on a successful response.

| Field | Type | Description |
|---|---|---|
| `key` | string | Name of the key that was pressed (e.g. `"space"`, `"left"`) |
| `rt_ms` | number | Reaction time in milliseconds |
| `rt_ns` | integer | Reaction time in nanoseconds |

---

## Functions

### `Trial.show(stim, ms?: number) -> Instant | nil`

Sends `stim` to the renderer and waits for the display flip. If `ms` is provided and greater than 0, the function additionally sleeps until `ms` milliseconds after the flip before returning.

Returns an `Instant` representing the flip time, or `nil` if no renderer is attached.

```lua
-- show fixation cross, wait 500 ms from flip
local flip = Trial.show(Stim.fixation(), 500)
```

---

### `Trial.blank(ms?: number) -> Instant | nil`

Clears the display and waits for the flip. Behaves identically to `Trial.show` but presents a blank/cleared screen.

```lua
-- blank screen, wait 200 ms from flip
local flip = Trial.blank(200)
```

---

### `Trial.preload_image(path: string)`

Pre-loads an image into GPU memory so that the first `Trial.show` call using it does not incur a loading delay. Raises an error if no renderer is attached.

```lua
Trial.preload_image("stimuli/face_01.png")
```

---

### `Trial.wait_key(opts?: table) -> KeyResponse | nil`

Arms a response window and blocks until a key is pressed or the timeout elapses.

Returns a `KeyResponse` on a valid response, or `nil` on timeout.

**Options table:**

| Key | Type | Description |
|---|---|---|
| `keys` | `string[]` | Accepted key names. If empty or omitted, any key is accepted. |
| `timeout` | number | Maximum wait time in milliseconds. If omitted, waits indefinitely. |

```lua
local resp = Trial.wait_key({ keys = {"f", "j"}, timeout = 2000 })
if resp then
    print(resp.key, resp.rt_ms)
end
```

---

### `Trial.next() -> integer`

Increments the internal trial counter by 1 and returns the new value. Call this at the start of each trial to keep the counter in sync with recorded data.

```lua
local trial_n = Trial.next()
```

---

### `Trial.set_block(n: integer)`

Sets the current block index to `n`. Used to tag recorded data with the correct block number.

```lua
Trial.set_block(2)
```

---

### `Trial.trial_index() -> integer`

Returns the current trial index without incrementing it.

```lua
local i = Trial.trial_index()
```