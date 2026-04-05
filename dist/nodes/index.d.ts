import type { RunContext } from '../scheduler/index.js';
export declare abstract class Node {
    abstract run(ctx: RunContext): Promise<void>;
}
export declare class SequenceNode extends Node {
    readonly children: readonly Node[];
    constructor(children: readonly Node[]);
    run(ctx: RunContext): Promise<void>;
}
export declare class RepeatNode extends Node {
    readonly child: Node;
    readonly count: number;
    readonly shuffle: boolean;
    constructor(child: Node, count: number, shuffle?: boolean);
    run(ctx: RunContext): Promise<void>;
}
export declare class FunctionNode extends Node {
    private readonly fn;
    constructor(fn: (ctx: RunContext) => Promise<void>);
    run(ctx: RunContext): Promise<void>;
}
//# sourceMappingURL=index.d.ts.map