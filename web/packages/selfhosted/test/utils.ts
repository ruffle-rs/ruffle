import path from "path";
import { expect } from "chai";
import { PublicAPI, RufflePlayer } from "ruffle-core";

declare global {
    interface Window {
        ruffleErrors: ErrorEvent[];
    }
}

declare module "ruffle-core" {
    interface RufflePlayer {
        __ruffle_log__: string;
    }
}

async function isRuffleLoaded(browser: WebdriverIO.Browser) {
    return await browser.execute(
        () =>
            window !== undefined &&
            window.RufflePlayer !== undefined &&
            window.RufflePlayer.invoked,
    );
}

async function waitForRuffle(browser: WebdriverIO.Browser) {
    await browser.waitUntil(async () => await isRuffleLoaded(browser), {
        timeoutMsg: "Expected Ruffle to load",
    });
    await throwIfError(browser);
}

async function setupErrorHandler(browser: WebdriverIO.Browser) {
    await browser.execute(() => {
        window.ruffleErrors = [];
        window.addEventListener("error", (error) => {
            window.ruffleErrors.push(error);
        });
    });
}

async function hasError(browser: WebdriverIO.Browser) {
    return await browser.execute(
        () => window.ruffleErrors && window.ruffleErrors.length > 0,
    );
}

async function throwIfError(browser: WebdriverIO.Browser) {
    return await browser.execute(() => {
        if (window.ruffleErrors && window.ruffleErrors.length > 0) {
            throw window.ruffleErrors[0];
        }
    });
}

async function injectRuffle(browser: WebdriverIO.Browser) {
    await setupErrorHandler(browser);
    await browser.execute(() => {
        const script = document.createElement("script");
        script.type = "text/javascript";
        script.src = "/dist/ruffle.js";
        document.head.appendChild(script);
    });
    await throwIfError(browser);
}

async function playAndMonitor(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
    expectedOutput: string | undefined = undefined,
) {
    await throwIfError(browser);

    // TODO: better way to test for this in the API
    await browser.waitUntil(
        async () =>
            (await hasError(browser)) ||
            // @ts-expect-error TS2341
            (await browser.execute((player) => player.instance, player)),
        {
            timeoutMsg: "Expected player to have initialized",
        },
    );

    await browser.execute((playerElement) => {
        // https://github.com/webdriverio/webdriverio/issues/6486
        const player = playerElement as unknown as RufflePlayer;
        player.__ruffle_log__ = "";
        player.traceObserver = (msg) => {
            player.__ruffle_log__ += msg + "\n";
        };
        player.play();
    }, player);

    if (expectedOutput === undefined) {
        expectedOutput = "Hello from Flash!\n";
    }

    const actualOutput = await getTraceOutput(browser, player);
    expect(actualOutput).to.eql(expectedOutput);
}

async function getTraceOutput(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
) {
    // Await any trace output
    await browser.waitUntil(
        async () => {
            return (
                (await browser.execute((player) => {
                    // https://github.com/webdriverio/webdriverio/issues/6486
                    return (player as unknown as RufflePlayer).__ruffle_log__;
                }, player)) !== ""
            );
        },
        {
            timeoutMsg: "Expected Ruffle to trace a message",
        },
    );

    // Get the output, and replace it with an empty string for any future test
    return await browser.execute((playerElement) => {
        // https://github.com/webdriverio/webdriverio/issues/6486
        const player = playerElement as unknown as RufflePlayer;
        const log = player.__ruffle_log__;
        player.__ruffle_log__ = "";
        return log;
    }, player);
}

async function injectRuffleAndWait(browser: WebdriverIO.Browser) {
    await injectRuffle(browser);
    await waitForRuffle(browser);
}

async function openTest(
    browser: WebdriverIO.Browser,
    absoluteDir: string,
    filename: string | undefined = undefined,
) {
    const dirname = path.basename(absoluteDir);
    if (filename === undefined) {
        filename = "index.html";
    }
    await browser.url(
        `http://localhost:4567/test/polyfill/${dirname}/${filename}`,
    );
}

/** Test set-up for JS API testing. */
function jsApiBefore(swf: string | undefined = undefined) {
    let player = null;

    before("Loads the test", async () => {
        await browser.url("http://localhost:4567/test_assets/js_api.html");

        await injectRuffleAndWait(browser);

        player = (await browser.execute(() => {
            const ruffle = (window.RufflePlayer as PublicAPI).newest();
            const player = ruffle!.createPlayer();
            const container = document.getElementById("test-container");
            container!.appendChild(player);
            return player;
            // https://github.com/webdriverio/webdriverio/issues/6486
        })) as unknown as WebdriverIO.Element;

        if (swf) {
            await browser.execute(
                async (player, swf) => {
                    // https://github.com/webdriverio/webdriverio/issues/6486
                    await (player as unknown as RufflePlayer).load(swf);
                },
                player,
                swf,
            );
            await playAndMonitor(browser, player);
        }
    });
}

export {
    isRuffleLoaded,
    waitForRuffle,
    playAndMonitor,
    injectRuffle,
    injectRuffleAndWait,
    openTest,
    setupErrorHandler,
    jsApiBefore,
    getTraceOutput,
};
