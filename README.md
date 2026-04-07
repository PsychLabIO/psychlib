# BrainLib

A TypeScript library for building browser-based psychology experiments. Provides a frame-loop runtime, stimulus rendering, keyboard response collection, and trial data recording.

## Installation

```bash
npm install brainlib
```

## Quick start

```typescript
import {
  Canvas2DRenderer, runBrowser,
  SequenceNode, RepeatNode, FunctionNode,
  Stim,
} from 'brainlib';

const canvas = document.getElementById('canvas') as HTMLCanvasElement;
const renderer = new Canvas2DRenderer(canvas);

const fixation = Stim.fixation(renderer);
const blank    = Stim.blank(renderer);
const target   = Stim.text(renderer, 'Press SPACE');

const trial = new FunctionNode(async (ctx) => {
  await ctx.show(fixation, 500);
  const response = await ctx.waitResponse(target, ['Space'], 2000);
  ctx.data.record({ rt: response?.timestamp ?? null, key: response?.key ?? null });
  await ctx.show(blank, 500);
});

const experiment = new SequenceNode([new RepeatNode(trial, 10)]);

const data = await runBrowser(renderer, experiment, {
  participantId: 'p01',
  experimentId:  'demo',
});

await data.save('/api/results');
```

## Core concepts

### Nodes

Experiments are built as a tree of `Node` objects.

| Class | Description |
|---|---|
| `SequenceNode(children)` | Runs children in order |
| `RepeatNode(child, count, shuffle?)` | Repeats a node N times; optionally shuffles |
| `FunctionNode(fn)` | Wraps an async function as a node |

Implement `abstract class Node` to create custom node types.

### RunContext

Passed to every node's `run(ctx)` call. Provides the three main experiment affordances:

```typescript
// Display a stimulus for a fixed duration
await ctx.show(renderFn, durationMs);

// Display a stimulus and wait for a keypress (returns null on timeout)
const response = await ctx.waitResponse(renderFn, ['ArrowLeft', 'ArrowRight'], timeoutMs);

ctx.renderer  // IRenderer - draw directly if needed
ctx.clock     // Clock - experiment-relative timestamps in ms
ctx.data      // DataWriter - record trial data
```

### Stimuli (`Stim.*`)

Factory functions that return `RenderFn` values for use with `ctx.show` / `ctx.waitResponse`.

```typescript
Stim.fixation(renderer, { size?, thickness?, color? })
Stim.blank(renderer, color?)
Stim.text(renderer, content, { x?, y?, size?, color?, align?, baseline? })
Stim.rect(renderer, x, y, w, h, color, cornerRadius?)
Stim.circle(renderer, cx, cy, radius, color)
Stim.image(renderer, src, x, y, w, h)
Stim.compose(...fns)  // layer multiple stimuli
```

### Clock

High-resolution experiment clock (wraps `performance.now`). Time zero is when the `Clock` was constructed (i.e. when `runBrowser` was called).

```typescript
ctx.clock.now()          // Ms since experiment start
ctx.clock.elapsed(since) // Ms since a recorded timestamp
```

### DataWriter

Accumulates trial records during the experiment.

```typescript
ctx.data.record({ condition: 'congruent', rt: 342, correct: true });

const { header, trials } = ctx.data.snapshot(); // inspect in-memory
await ctx.data.save('/api/results');             // POST as JSON
```

Each record is stamped with `_wallTime` (Unix ms) automatically. The session header includes `participantId`, `experimentId`, `startTime`, and any extra fields passed to `runBrowser`.

### Renderer

`Canvas2DRenderer` is the built-in renderer. Pass an `HTMLCanvasElement` to its constructor. It implements `IRenderer`, so you can substitute a custom renderer (e.g. WebGL) by implementing that interface.

```typescript
const renderer = new Canvas2DRenderer(canvas);
await renderer.preload(['/img/stim1.png', '/img/stim2.png']); // optional pre-load
```

## Building

```bash
npm run build          # compile TypeScript to dist/
npm run watch          # watch mode
npm run check          # type-check only, no output
```

## License

MIT
