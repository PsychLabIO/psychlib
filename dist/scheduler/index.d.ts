import type { Clock } from '../clock/index.js';
import type { IRenderer } from '../renderer/IRenderer.js';
import type { DataWriter } from '../data/index.js';
import type { Response, KeyCode } from '../io/index.js';
type RenderFn = () => void;
export declare class RunContext {
    readonly renderer: IRenderer;
    readonly clock: Clock;
    readonly data: DataWriter;
    private renderFn;
    private waiter;
    constructor(renderer: IRenderer, clock: Clock, data: DataWriter);
    tick(): void;
    attachBrowserInput(): void;
    show(render: RenderFn, durationMs: number): Promise<void>;
    waitResponse(render: RenderFn, keys: KeyCode[], timeoutMs: number): Promise<Response | null>;
}
export {};
//# sourceMappingURL=index.d.ts.map