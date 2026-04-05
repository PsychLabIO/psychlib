export class Node {
}
export class SequenceNode extends Node {
    children;
    constructor(children) {
        super();
        this.children = children;
    }
    async run(ctx) {
        for (const child of this.children) {
            await child.run(ctx);
        }
    }
}
export class RepeatNode extends Node {
    child;
    count;
    shuffle;
    constructor(child, count, shuffle = false) {
        super();
        this.child = child;
        this.count = count;
        this.shuffle = shuffle;
    }
    async run(ctx) {
        const indices = Array.from({ length: this.count }, (_, i) => i);
        if (this.shuffle)
            fisherYates(indices);
        for (const _i of indices) {
            await this.child.run(ctx);
        }
    }
}
export class FunctionNode extends Node {
    fn;
    constructor(fn) {
        super();
        this.fn = fn;
    }
    run(ctx) {
        return this.fn(ctx);
    }
}
function fisherYates(arr) {
    for (let i = arr.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [arr[i], arr[j]] = [arr[j], arr[i]];
    }
}
//# sourceMappingURL=index.js.map