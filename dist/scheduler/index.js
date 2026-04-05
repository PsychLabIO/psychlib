import { Color } from '../renderer/types.js';
export class RunContext {
    renderer;
    clock;
    data;
    renderFn = () => { };
    waiter = null;
    constructor(renderer, clock, data) {
        this.renderer = renderer;
        this.clock = clock;
        this.data = data;
    }
    tick() {
        const now = this.clock.now();
        this.renderer.beginFrame(now);
        this.renderer.clear(Color.black);
        this.renderFn();
        this.renderer.endFrame();
        if (this.waiter === null)
            return;
        if (now >= this.waiter.deadline) {
            const w = this.waiter;
            this.waiter = null;
            if (w.kind === 'response') {
                w.resolve(null); // timeout
            }
            else {
                w.resolve();
            }
        }
    }
    attachBrowserInput() {
        window.addEventListener('keydown', (e) => {
            if (this.waiter?.kind !== 'response')
                return;
            if (!this.waiter.keys.has(e.code))
                return;
            const w = this.waiter;
            this.waiter = null;
            w.resolve({ key: e.code, timestamp: this.clock.now() });
        });
    }
    show(render, durationMs) {
        this.renderFn = render;
        const deadline = this.clock.now() + durationMs;
        return new Promise(resolve => {
            this.waiter = { kind: 'duration', deadline, resolve };
        });
    }
    waitResponse(render, keys, timeoutMs) {
        this.renderFn = render;
        const deadline = this.clock.now() + timeoutMs;
        return new Promise(resolve => {
            this.waiter = { kind: 'response', keys: new Set(keys), deadline, resolve };
        });
    }
}
//# sourceMappingURL=index.js.map