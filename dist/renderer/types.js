export const Color = {
    white: { r: 1, g: 1, b: 1, a: 1 },
    black: { r: 0, g: 0, b: 0, a: 1 },
    gray: { r: 0.5, g: 0.5, b: 0.5, a: 1 },
    red: { r: 1, g: 0, b: 0, a: 1 },
    green: { r: 0, g: 1, b: 0, a: 1 },
    blue: { r: 0, g: 0, b: 1, a: 1 },
    rgb(r, g, b) {
        return { r, g, b, a: 1 };
    },
    rgba(r, g, b, a) {
        return { r, g, b, a };
    },
    rgb255(r, g, b) {
        return { r: r / 255, g: g / 255, b: b / 255, a: 1 };
    },
};
//# sourceMappingURL=types.js.map