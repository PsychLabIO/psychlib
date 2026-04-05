import type { IRenderer } from "../renderer/IRenderer.js";
import type { Color, TextOptions } from "../renderer/types.js";
export type RenderFn = () => void;
export declare function fixation(renderer: IRenderer, opts?: {
    size?: number;
    thickness?: number;
    color?: Color;
}): RenderFn;
export declare function blank(renderer: IRenderer, color?: Color): RenderFn;
export declare function text(renderer: IRenderer, content: string, opts?: Partial<TextOptions> & {
    x?: number;
    y?: number;
}): RenderFn;
export declare function rect(renderer: IRenderer, x: number, y: number, w: number, h: number, color: Color, cornerRadius?: number): RenderFn;
export declare function circle(renderer: IRenderer, cx: number, cy: number, radius: number, color: Color): RenderFn;
export declare function image(renderer: IRenderer, src: string, x: number, y: number, w: number, h: number): RenderFn;
export declare function compose(...fns: RenderFn[]): RenderFn;
//# sourceMappingURL=index.d.ts.map