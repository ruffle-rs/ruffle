import init, { ActiveTest, get_test } from "../../build/web_test_runner";
import { RunnerToOrchestratorMessage } from "../worker_api.tsx";

function sendMessage(message: RunnerToOrchestratorMessage) {
    postMessage(message);
}

function sleep(duration: number) {
    return new Promise((resolve) => {
        setTimeout(resolve, duration);
    });
}

async function runTest(name: string): Promise<RunnerToOrchestratorMessage> {
    await init();
    const test = catchPanic(() => get_test(name));
    if (test === undefined) {
        return {
            needsRestart: false,
            result: { state: "skipped", result: "Unknown test" },
            type: "test_finished",
        };
    }

    // [NA] Rule of thumb: panics need a restarted worker. If in catch block, `needsRestart: true`

    // Pull this out just in case we panic and get into a weird post-panic-state
    const knownFailure = test.known_failure;

    let activeTest: ActiveTest;
    try {
        activeTest = await catchPanic(() => test.start());
    } catch (e) {
        const error = e as Error; // Guaranteed by `catchPanic`
        return {
            needsRestart: true,
            type: "test_finished",
            result: {
                result: error.message,
                stack: error.stack,
                state: "failed",
            },
        };
    }

    while (!activeTest.finished) {
        try {
            catchPanic(() => activeTest.tick());
            await sleep(activeTest.sleep);
            catchPanic(() => activeTest.run());
        } catch (e) {
            freeIfPossible(activeTest);
            if (knownFailure) {
                return {
                    needsRestart: true,
                    type: "test_finished",
                    result: { state: "success" },
                };
            } else {
                const error = e as Error; // Guaranteed by `catchPanic`
                return {
                    needsRestart: true,
                    type: "test_finished",
                    result: {
                        result: error.message,
                        stack: error.stack,
                        state: "failed",
                    },
                };
            }
        }
    }

    let response: RunnerToOrchestratorMessage;

    if (activeTest.skipped) {
        response = {
            needsRestart: false,
            type: "test_finished",
            result: { state: "skipped" },
        };
    } else if (activeTest.error) {
        if (knownFailure) {
            response = {
                needsRestart: false,
                type: "test_finished",
                result: { state: "success" },
            };
        } else {
            response = {
                needsRestart: false,
                type: "test_finished",
                result: {
                    state: "failed",
                    result: activeTest.error,
                },
            };
        }
    } else {
        if (knownFailure) {
            response = {
                needsRestart: false,
                type: "test_finished",
                result: {
                    state: "failed",
                    result: `${name} was known to be failing, but now passes successfully. Please update it and remove \`known_failure = true\`!`,
                },
            };
        } else {
            response = {
                needsRestart: false,
                type: "test_finished",
                result: { state: "success" },
            };
        }
    }

    freeIfPossible(activeTest);
    return response;
}

addEventListener("message", (event) => {
    const name = event.data as string;
    runTest(name)
        .then((result) => sendMessage(result))
        .catch((e) => {
            setTimeout(function () {
                throw catchPanic(() => e);
            });
        });
});

declare global {
    interface WorkerGlobalScope {
        onPanic?: (error: Error) => void;
    }
}

// When rust panics, it'll trigger our `onPanic` method below and *then* throw in the calling JS.
// Store the panic here, and use it in the catch block to be able to see what it was.
let panic: Error | null = null;

self.onPanic = (error: Error) => {
    panic = error;
};

/**
 * Calls the given function, translating any panics to the real panic message rather than just "Unreachable executed"
 * @param f Function to call
 * @throws {Error} Either the true panic cause, or an {Error} wrapping what we know about the error
 */
function catchPanic<T>(f: () => T): T {
    try {
        return f();
    } catch (e) {
        if (panic) {
            throw panic;
        } else if (e instanceof Error) {
            throw e;
        } else {
            throw new Error(e as string);
        }
    }
}

function freeIfPossible(object: { free: () => void }) {
    try {
        object.free();
    } catch (error) {
        console.error("Error trying to free rust object", error);
    }
}
