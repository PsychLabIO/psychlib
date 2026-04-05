import { DataWriter } from "../data/index.js";
import type { Node } from "../nodes/index.js";
import type { IRenderer } from "../renderer/IRenderer.js";
export interface ExperimentConfig {
    participantId: string;
    experimentId: string;
    [key: string]: unknown;
}
export declare function runBrowser(renderer: IRenderer, root: Node, config: ExperimentConfig): Promise<DataWriter>;
//# sourceMappingURL=browser.d.ts.map