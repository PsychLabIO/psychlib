export class DataWriter {
    header;
    trials = [];
    constructor(header) {
        this.header = header;
    }
    record(trial) {
        this.trials.push({ ...trial, _wallTime: Date.now() });
    }
    snapshot() {
        return { header: this.header, trials: [...this.trials] };
    }
    async save(endpoint) {
        const res = await fetch(endpoint, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(this.snapshot()),
        });
        if (!res.ok) {
            throw new Error(`Failed to save data: ${res.status} ${res.statusText}`);
        }
    }
}
//# sourceMappingURL=index.js.map