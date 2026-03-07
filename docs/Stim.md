# Stim

Stimulus constructors and color utilities.

`Stim` is available as a global in every experiment script. All constructor functions return an opaque `Stimulus` value that can be passed to the [`Stimulus`](nodes.md#stimulus) node's `stim` option.

---

## Stimulus constructors

### `Stim.text(content: string, opts?: table) -> Stimulus`

A text stimulus rendered at the given position.

**Options:**

| Key | Type | Default | Description |
|---|---|---|---|
| `size` | number | - | Font size (must be > 0) |
| `color` | Color or string | white | Text color |
| `align` | string | - | `"left"`, `"center"`, or `"right"` |
| `font` | string | - | Font family name |
| `x` | number | 0.0 | Horizontal position (normalised screen coords) |
| `y` | number | 0.0 | Vertical position (normalised screen coords) |

```lua
local prompt = Stim.text("Press F or J", { size = 0.06, color = "white", align = "center" })
```

---

### `Stim.fixation(opts?: table) -> Stimulus`

A fixation cross.

**Options:**

| Key | Type | Default | Description |
|---|---|---|---|
| `color` | Color or string | white | Cross color |
| `arm_len` | number | 0.03 | Half-length of each arm (normalised) |
| `thickness` | number | 0.005 | Line thickness (normalised) |

```lua
local fix = Stim.fixation({ color = "white" })
```

---

### `Stim.rect(cx, cy, hw, hh: number, color?: Color) -> Stimulus`

A filled rectangle.

| Parameter | Description |
|---|---|
| `cx`, `cy` | Centre position (normalised screen coords) |
| `hw`, `hh` | Half-width and half-height (normalised) |
| `color` | Fill color (default: white) |

```lua
local box = Stim.rect(0, 0, 0.1, 0.05, Stim.color("red"))
```

---

### `Stim.blank(color?: Color | string) -> Stimulus`

A full-screen blank stimulus. Defaults to black.

```lua
local blank = Stim.blank()
local gray_blank = Stim.blank("gray")
```

---

### `Stim.image(path: string, opts?: table) -> Stimulus`

An image loaded from disk.

**Options:**

| Key | Type | Default | Description |
|---|---|---|---|
| `cx` | number | 0.0 | Centre X (normalised) |
| `cy` | number | 0.0 | Centre Y (normalised) |
| `hw` | number | 0.5 | Half-width (normalised) |
| `hh` | number | 0.5 | Half-height (normalised) |
| `tint` | Color or string | white | Multiplicative tint |

```lua
local face = Stim.image("stimuli/face_01.png", { hw = 0.3, hh = 0.3 })
```

To avoid a loading delay on first presentation, preload images at the start of the experiment using `Stim.preload`:

```lua
Stim.preload("stimuli/face_01.png")
```

---

### `Stim.composite(parts: Stimulus[]) -> Stimulus`

Combines multiple stimuli into a single stimulus drawn in order. `parts` must be a non-empty sequence of `Stimulus` values.

```lua
local display = Stim.composite({
    Stim.fixation(),
    Stim.text("+5", { color = "yellow", y = 0.3 }),
})
```

---

## Color utilities

### `Stim.color(spec: string) -> Color`

Parses a color from a string and returns a `Color` value. Accepts:

- Hex strings: `"#RRGGBB"` or `"#RRGGBBAA"`
- Named colors: `"white"`, `"black"`, `"red"`, `"green"`, `"blue"`, `"gray"` / `"grey"`, `"yellow"`, `"cyan"` / `"aqua"`, `"magenta"`, `"orange"`, `"purple"`, `"transparent"`

```lua
local c = Stim.color("#ff6600")
```

---

### `Stim.rgb(r, g, b: number) -> Color`

Creates a color from 0–255 RGB components (fully opaque).

```lua
local c = Stim.rgb(255, 128, 0)
```

---

### `Stim.rgba(r, g, b, a: number) -> Color`

Creates a color from 0–255 RGBA components.

```lua
local c = Stim.rgba(255, 128, 0, 200)
```

---

## Image preloading

### `Stim.preload(path: string)`

Pre-loads an image into GPU memory so that the first `Stimulus` node using it does not incur a loading delay. Raises an error if no renderer is attached (e.g. in headless tests).

```lua
-- Preload at the top of the script before any trials run
Stim.preload("stimuli/face_01.png")
Stim.preload("stimuli/house_01.png")
```