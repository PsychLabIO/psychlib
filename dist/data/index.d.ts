export interface SessionHeader {
    participantId: string;
    experimentId: string;
    startTime: string;
    [key: string]: unknown;
}
export type TrialRecord = Record<string, unknown>;
export declare class DataWriter {
    readonly header: SessionHeader;
    private readonly trials;
    constructor(header: SessionHeader);
    record(trial: TrialRecord): void;
    snapshot(): {
        header: SessionHeader;
        trials: readonly TrialRecord[];
    };
    save(endpoint: string): Promise<void>;
}
//# sourceMappingURL=index.d.ts.map