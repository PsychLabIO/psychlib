import { Color as C } from "../renderer/types.js";
export function fixation(renderer, opts = {}) {
    const { size = 20, thickness = 2, color = C.white } = opts;
    const cx = renderer.width / 2;
    const cy = renderer.height / 2;
    return () => renderer.drawFixation(cx, cy, size, thickness, color);
}
export function blank(renderer, color = C.gray) {
    return () => renderer.clear(color);
}
export function text(renderer, content, opts = {}) {
    const x = opts.x ?? renderer.width / 2;
    const y = opts.y ?? renderer.height / 2;
    const resolved = {
        size: opts.size ?? 32,
        color: opts.color ?? C.white,
        align: opts.align ?? "center",
        baseline: opts.baseline ?? "middle",
    };
    return () => renderer.drawText(x, y, content, resolved);
}
export function rect(renderer, x, y, w, h, color, cornerRadius = 0) {
    return () => renderer.drawRect(x, y, w, h, color, cornerRadius);
}
export function circle(renderer, cx, cy, radius, color) {
    return () => renderer.drawCircle(cx, cy, radius, color);
}
export function image(renderer, src, x, y, w, h) {
    return () => renderer.drawImage(x, y, w, h, src);
}
export function compose(...fns) {
    return () => {
        for (const fn of fns)
            fn();
    };
}
//# sourceMappingURL=index.js.map