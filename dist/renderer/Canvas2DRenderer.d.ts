import type { Color, TextOptions } from "./types.js";
import type { IRenderer } from "./IRenderer.js";
export declare class Canvas2DRenderer implements IRenderer {
    private readonly canvas;
    private readonly ctx;
    private readonly images;
    constructor(canvas: HTMLCanvasElement);
    get width(): number;
    get height(): number;
    /** No-op; requestAnimationFrame already provides vsync. */
    beginFrame(_timestamp: number): void;
    /** No-op; the browser composites automatically. */
    endFrame(): void;
    clear(color: Color): void;
    drawRect(x: number, y: number, w: number, h: number, color: Color, cornerRadius?: number): void;
    drawCircle(cx: number, cy: number, radius: number, color: Color): void;
    drawText(x: number, y: number, text: string, opts: TextOptions): void;
    drawFixation(x: number, y: number, size: number, thickness: number, color: Color): void;
    drawImage(x: number, y: number, w: number, h: number, src: string): void;
    preload(srcs: string[]): Promise<void[]>;
}
//# sourceMappingURL=Canvas2DRenderer.d.ts.map