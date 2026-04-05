import { Clock } from "../clock/index.js";
import { DataWriter } from "../data/index.js";
import { RunContext } from "../scheduler/index.js";
export async function runBrowser(renderer, root, config) {
    const { participantId, experimentId, ...extra } = config;
    const clock = new Clock();
    const data = new DataWriter({
        participantId,
        experimentId,
        startTime: new Date().toISOString(),
        ...extra,
    });
    const ctx = new RunContext(renderer, clock, data);
    ctx.attachBrowserInput();
    let done = false;
    const finished = root.run(ctx).then(() => {
        done = true;
    });
    await new Promise((resolveLoop) => {
        function frame() {
            ctx.tick();
            if (done) {
                resolveLoop();
            }
            else {
                requestAnimationFrame(frame);
            }
        }
        requestAnimationFrame(frame);
    });
    await finished;
    return data;
}
//# sourceMappingURL=browser.js.map