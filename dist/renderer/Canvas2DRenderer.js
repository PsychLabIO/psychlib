function css(c) {
    return `rgba(${Math.round(c.r * 255)},${Math.round(c.g * 255)},${Math.round(c.b * 255)},${c.a})`;
}
export class Canvas2DRenderer {
    canvas;
    ctx;
    images = new Map();
    constructor(canvas) {
        this.canvas = canvas;
        const ctx = canvas.getContext("2d");
        if (ctx === null)
            throw new Error("Canvas2DRenderer: failed to acquire 2D context");
        this.ctx = ctx;
    }
    get width() {
        return this.canvas.width;
    }
    get height() {
        return this.canvas.height;
    }
    /** No-op; requestAnimationFrame already provides vsync. */
    beginFrame(_timestamp) { }
    /** No-op; the browser composites automatically. */
    endFrame() { }
    clear(color) {
        this.ctx.fillStyle = css(color);
        this.ctx.fillRect(0, 0, this.width, this.height);
    }
    drawRect(x, y, w, h, color, cornerRadius = 0) {
        this.ctx.fillStyle = css(color);
        if (cornerRadius > 0) {
            this.ctx.beginPath();
            this.ctx.roundRect(x, y, w, h, cornerRadius);
            this.ctx.fill();
        }
        else {
            this.ctx.fillRect(x, y, w, h);
        }
    }
    drawCircle(cx, cy, radius, color) {
        this.ctx.fillStyle = css(color);
        this.ctx.beginPath();
        this.ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        this.ctx.fill();
    }
    drawText(x, y, text, opts) {
        const { size, color, align = "center", baseline = "middle" } = opts;
        this.ctx.fillStyle = css(color);
        this.ctx.font = `${size}px sans-serif`;
        this.ctx.textAlign = align;
        this.ctx.textBaseline = baseline;
        const lines = text.split("\n");
        if (lines.length === 1) {
            this.ctx.fillText(text, x, y);
            return;
        }
        const lineHeight = size * 1.4;
        const startY = baseline === "middle"
            ? y - ((lines.length - 1) * lineHeight) / 2
            : y;
        for (let i = 0; i < lines.length; i++) {
            this.ctx.fillText(lines[i], x, startY + i * lineHeight);
        }
    }
    drawFixation(x, y, size, thickness, color) {
        const half = size / 2;
        this.ctx.strokeStyle = css(color);
        this.ctx.lineWidth = thickness;
        this.ctx.beginPath();
        this.ctx.moveTo(x - half, y);
        this.ctx.lineTo(x + half, y);
        this.ctx.moveTo(x, y - half);
        this.ctx.lineTo(x, y + half);
        this.ctx.stroke();
    }
    drawImage(x, y, w, h, src) {
        let img = this.images.get(src);
        if (img === undefined) {
            img = new Image();
            img.src = src;
            this.images.set(src, img);
        }
        if (img.complete && img.naturalWidth > 0) {
            this.ctx.drawImage(img, x, y, w, h);
        }
    }
    preload(srcs) {
        return Promise.all(srcs.map((src) => new Promise((resolve, reject) => {
            const img = new Image();
            img.onload = () => {
                this.images.set(src, img);
                resolve();
            };
            img.onerror = () => reject(new Error(`Failed to load image: ${src}`));
            img.src = src;
        })));
    }
}
//# sourceMappingURL=Canvas2DRenderer.js.map