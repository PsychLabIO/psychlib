export interface Color {
    readonly r: number;
    readonly g: number;
    readonly b: number;
    readonly a: number;
}
export declare const Color: {
    readonly white: Color;
    readonly black: Color;
    readonly gray: Color;
    readonly red: Color;
    readonly green: Color;
    readonly blue: Color;
    readonly rgb: (r: number, g: number, b: number) => Color;
    readonly rgba: (r: number, g: number, b: number, a: number) => Color;
    readonly rgb255: (r: number, g: number, b: number) => Color;
};
export interface TextOptions {
    size: number;
    color: Color;
    align?: 'left' | 'center' | 'right';
    baseline?: 'top' | 'middle' | 'bottom';
}
//# sourceMappingURL=types.d.ts.map