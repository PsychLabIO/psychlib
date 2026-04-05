export class Clock {
    origin;
    constructor() {
        this.origin = performance.now();
    }
    now() {
        return performance.now() - this.origin;
    }
    elapsed(since) {
        return this.now() - since;
    }
}
//# sourceMappingURL=index.js.map