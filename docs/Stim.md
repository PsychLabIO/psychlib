# Stim

Stimulus constructors and color utilities.

`Stim` is available as a global in every experiment script. All constructor functions return an opaque `Stimulus` value that can be passed to the [`Stimulus`](nodes.md#stimulus) node's `stim` option.

---

## Coordinate system

Positions use **normalised 0–1 coordinates**, top-left origin, y increasing downward:

| Point | Meaning |
|---|---|
| `(0.0, 0.0)` | Top-left corner |
| `(0.5, 0.5)` | Centre of screen |
| `(1.0, 1.0)` | Bottom-right corner |

Sizes (`size`, `hw`, `hh`, `arm_len`, `thickness`) are in **pixels** and are independent of screen resolution.

---

## Stimulus constructors

### `Stim.text(content: string, opts?: table) -> Stimulus`

A text stimulus rendered at the given position.

**Options:**

| Key | Type | Default | Description |
|---|---|---|---|
| `size` | number | - | Font size in **pixels** (must be > 0) |
| `color` | Color or string | white | Text color |
| `align` | string | - | `"left"`, `"center"`, or `"right"` |
| `font` | string | - | Font family name |
| `x` | number | 0.5 | Horizontal position (0–1, left→right) |
| `y` | number | 0.5 | Vertical position (0–1, top→bottom) |

```lua
-- Centred text at default position (0.5, 0.5)
local prompt = Stim.text("Press F or J", { size = 24, color = "white", align = "center" })

-- Positioned in the upper third of the screen
local cue = Stim.text("FAST", { size = 20, color = "yellow", x = 0.5, y = 0.3 })
```

---

### `Stim.fixation(opts?: table) -> Stimulus`

A fixation cross.

> **Known issue:** The fixation stimulus currently renders as a rect. Fix pending.

**Options:**

| Key | Type | Default | Description |
|---|---|---|---|
| `color` | Color or string | white | Cross color |
| `arm_len` | number | ~2.8% of screen height | Half-length of each arm in **pixels** |
| `thickness` | number | ~0.4% of screen height (min 2) | Line thickness in **pixels** |

```lua
local fix = Stim.fixation({ color = "white", arm_len = 24, thickness = 4 })
```

---

### `Stim.rect(cx, cy, hw, hh: number, color?: Color) -> Stimulus`

A filled rectangle.

| Parameter | Description |
|---|---|
| `cx`, `cy` | Centre position (0–1, top-left origin) |
| `hw`, `hh` | Half-width and half-height in **pixels** |
| `color` | Fill color (default: white) |

```lua
-- A 200×80px white box at centre-screen
local box = Stim.rect(0.5, 0.5, 100, 40)

-- A red box in the top-right quadrant
local marker = Stim.rect(0.75, 0.25, 30, 30, Stim.color("red"))
```

---

### `Stim.blank(color?: Color | string) -> Stimulus`

A full-screen blank stimulus. Defaults to black.

> **Known issue:** Currently non-functional. Fix pending.

```lua
local blank      = Stim.blank()
local gray_blank = Stim.blank("gray")
```

---

### `Stim.image(path: string, opts?: table) -> Stimulus`

An image loaded from disk.

**Options:**

| Key | Type | Default | Description |
|---|---|---|---|
| `cx` | number | 0.5 | Centre X (0–1, left→right) |
| `cy` | number | 0.5 | Centre Y (0–1, top→bottom) |
| `hw` | number | 200 | Half-width in **pixels** |
| `hh` | number | 200 | Half-height in **pixels** |
| `tint` | Color or string | white | Multiplicative tint |

```lua
-- 400×400px image at centre
local face = Stim.image("stimuli/face_01.png", { hw = 200, hh = 200 })

-- 300×200px image in the lower half
local scene = Stim.image("stimuli/scene.png", { cx = 0.5, cy = 0.7, hw = 150, hh = 100 })
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
    Stim.text("+5", { size = 28, color = "yellow", y = 0.65 }),
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