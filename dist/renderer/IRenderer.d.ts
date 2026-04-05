import type { Color, TextOptions } from "./types.js";
export interface IRenderer {
    readonly width: number;
    readonly height: number;
    beginFrame(timestamp: number): void;
    endFrame(): void;
    clear(color: Color): void;
    drawRect(x: number, y: number, w: number, h: number, color: Color, cornerRadius?: number): void;
    drawCircle(cx: number, cy: number, radius: number, color: Color): void;
    drawText(x: number, y: number, text: string, opts: TextOptions): void;
    drawFixation(x: number, y: number, size: number, thickness: number, color: Color): void;
    drawImage(x: number, y: number, w: number, h: number, src: string): void;
}
//# sourceMappingURL=IRenderer.d.ts.map