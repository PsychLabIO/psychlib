# Rand

Pseudo-random number generation and sequence utilities.

`Rand` is available as a global in every experiment script. The generator is seeded either from a fixed seed (for reproducible sessions) or from system entropy. The underlying algorithm is xorshift64.

---

## Functions

### `Rand.int(lo: integer, hi: integer) -> integer`

Returns a uniformly distributed random integer in the inclusive range `[lo, hi]`. Returns `lo` if `lo >= hi`.

```lua
local side = Rand.int(1, 2)   -- 1 or 2
```

---

### `Rand.float(lo: number, hi: number) -> number`

Returns a uniformly distributed random float in `[lo, hi)`.

```lua
local jitter = Rand.float(400, 600)   -- e.g. 537.2
```

---

### `Rand.shuffle(tbl: table) -> table`

Returns a new table containing the same elements as `tbl` in a random order (Fisher-Yates). The original table is not modified. Returns the original table unchanged if it is empty.

```lua
local order = Rand.shuffle({"A", "B", "C", "D"})
```

---

### `Rand.choice(tbl: table) -> any`

Returns a single randomly selected element from `tbl`. Returns `nil` if the table is empty.

```lua
local word = Rand.choice(word_list)
```

---

### `Rand.balanced_shuffle(items: table, n: integer) -> table`

Builds a sequence of length `n` from `items` such that each item appears as evenly as possible (cycling through items in order to fill `n` slots), then shuffles the result. Useful for constructing balanced trial lists.

```lua
-- 12 trials balanced across 3 conditions
local trials = Rand.balanced_shuffle({"congruent", "incongruent", "neutral"}, 12)
```

If `items` is empty or `n` is 0, returns an empty table.

---

## See also

The [`Shuffle(list)`](nodes.md#utility-functions) utility function in the stdlib is a thin wrapper around `Rand.shuffle` designed for use with `ForTrials`:

```lua
ForTrials(Shuffle(make_trials()), function(trial) ... end)
```