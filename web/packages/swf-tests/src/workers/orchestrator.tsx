import init, { list_tests, TestInfo } from "../../build/web_test_runner";
import {
    AppToOrchestratorMessage,
    OrchestratorToAppMessage,
    RendererRequirement,
    RunnerToOrchestratorMessage,
} from "../worker_api.tsx";

function sendMessage(message: OrchestratorToAppMessage) {
    postMessage(message);
}

interface Runner extends Omit<Worker, "postMessage"> {
    postMessage(command: string): void;
}

let allTests: TestInfo[] = [];
const testsToRun: string[] = [];
const runnerPool: Runner[] = [];

let numRunners = 0;

async function start() {
    await init();
    allTests = list_tests();
    sendMessage({
        type: "test_list",
        tests: allTests.map((test) => {
            return {
                name: test.name,
                ignored: test.ignored,
                known_failure: test.known_failure,
                should_run: test.should_run,
                wants_renderer: test.wants_renderer as RendererRequirement,
            };
        }),
    });
}
start().catch(console.error);

let running = false;
let totalQueued = 0;
let totalDone = 0;
let totalPass = 0;
let totalFail = 0;
let totalSkip = 0;

function createRunner() {
    const runner = new Worker(new URL("./runner.tsx", import.meta.url), {
        type: "module",
    }) as Runner;
    runnerPool.push(runner);
}

function setRunnerCount(newCount: number) {
    const delta = newCount - numRunners;
    console.log(`Setting runner count to ${newCount} from ${numRunners}`);
    if (delta > 0) {
        for (let i = 0; i < delta; i++) {
            createRunner();
        }
    } else if (delta < 0) {
        for (let i = 0; i < -delta; i++) {
            const runner = runnerPool.pop();
            if (runner) {
                runner.terminate();
            }
        }
        // TODO: If there's tests in progress, they might dangle some runners.
    }
    numRunners = newCount;
}

function tick() {
    running = false;
    while (runnerPool.length > 0 && testsToRun.length > 0) {
        const nextTest = testsToRun.shift()!;
        const runner = runnerPool.pop()!;
        console.log("Starting test", nextTest);
        sendMessage({
            type: "update_test_result",
            name: nextTest,
            result: {
                state: "running",
            },
        });
        let finished = false;
        runner.onmessage = (e) => {
            if (finished) return;
            finished = true;
            const message = e.data as RunnerToOrchestratorMessage;
            totalDone++;
            switch (message.result.state) {
                case "skipped":
                    totalSkip++;
                    break;
                case "failed":
                    totalFail++;
                    break;
                case "success":
                    totalPass++;
                    break;
            }
            sendMessage({
                type: "update_test_result",
                name: nextTest,
                result: message.result,
            });
            sendMessage({
                type: "progress_update",
                total: totalQueued,
                done: totalDone,
                failed: totalFail,
                passed: totalPass,
                skipped: totalSkip,
            });
            if (message.needsRestart) {
                console.error("Runner needs restart");
                runner.terminate();
                createRunner();
            } else {
                runnerPool.push(runner);
            }
            queueTick();
        };
        runner.onerror = (e) => {
            if (finished) return;
            finished = true;
            console.error("Runner failed", e.message);
            runner.terminate();
            createRunner();
            testsToRun.unshift(nextTest); // Push it at the front
            queueTick();
        };
        runner.postMessage(nextTest);
    }
}

function queueTick() {
    if (!running) {
        setTimeout(tick, 5);
        running = true;
    }
}

function queueTests(tests: string[]) {
    testsToRun.push(...tests);
    queueTick();
}

addEventListener("message", (event) => {
    const message = event.data as AppToOrchestratorMessage;
    switch (message.type) {
        case "start_test":
            if (testsToRun.length == 0) {
                // Reset these numbers when a *new* test starts and we finished last queue
                totalDone = 0;
                totalQueued = 0;
                totalFail = 0;
                totalPass = 0;
                totalSkip = 0;
            }
            totalQueued += message.names.length;
            sendMessage({
                type: "progress_update",
                total: totalQueued,
                done: totalDone,
                failed: totalFail,
                passed: totalPass,
                skipped: totalSkip,
            });
            queueTests(message.names);
            break;
        case "configure_workers":
            setRunnerCount(message.amount);
            break;
    }
});
