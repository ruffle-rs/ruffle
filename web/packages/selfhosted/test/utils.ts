import { expect } from "chai";
import { Player, PublicAPI } from "ruffle-core";

declare global {
    interface Window {
        ruffleErrors: ErrorEvent[];
    }
}

declare module "ruffle-core" {
    interface Player {
        __ruffle_log__: string;
    }
}

export async function isRuffleLoaded(browser: WebdriverIO.Browser) {
    return await browser.execute(
        () =>
            window !== undefined &&
            window.RufflePlayer !== undefined &&
            window.RufflePlayer.invoked,
    );
}

export async function isRufflePlayerLoaded(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
) {
    return (
        (await browser.execute(
            (player) =>
                // https://github.com/webdriverio/webdriverio/issues/6486
                // TODO: How can we import ReadyState enum?
                (player as unknown as Player).readyState,
            player,
        )) === 2
    );
}

export async function waitForRuffle(browser: WebdriverIO.Browser) {
    await browser.waitUntil(async () => await isRuffleLoaded(browser), {
        timeoutMsg: "Expected Ruffle to load",
    });
    await throwIfError(browser);
}

export async function setupErrorHandler(browser: WebdriverIO.Browser) {
    await browser.execute(() => {
        window.ruffleErrors = [];
        window.addEventListener("error", (error) => {
            window.ruffleErrors.push(error);
        });
    });
}

export async function hasError(browser: WebdriverIO.Browser) {
    return await browser.execute(
        () => window.ruffleErrors && window.ruffleErrors.length > 0,
    );
}

export async function throwIfError(browser: WebdriverIO.Browser) {
    return await browser.execute(() => {
        if (window.ruffleErrors && window.ruffleErrors.length > 0) {
            throw window.ruffleErrors[0];
        }
    });
}

export async function injectRuffle(browser: WebdriverIO.Browser) {
    await setupErrorHandler(browser);
    await browser.execute(() => {
        // Don't use autoplay by default, as we want to control loading in the tests
        window.RufflePlayer ??= {};
        window.RufflePlayer.config = {
            autoplay: "off",
            ...(window.RufflePlayer.config || {}),
        };
        const script = document.createElement("script");
        script.type = "text/javascript";
        script.src = "/dist/ruffle.js";
        document.head.appendChild(script);
    });
    await throwIfError(browser);
}

export async function playAndMonitor(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
    expectedOutput: string = "Hello from Flash!\n",
) {
    await throwIfError(browser);
    await waitForPlayerToLoad(browser, player);
    await setupAndPlay(browser, player);

    const actualOutput = await getTraceOutput(browser, player);
    expect(actualOutput).to.eql(expectedOutput);
}

export async function setupAndPlay(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
) {
    await browser.execute((playerElement) => {
        // https://github.com/webdriverio/webdriverio/issues/6486
        const player = playerElement as unknown as Player;
        player.__ruffle_log__ = "";
        player.traceObserver = (msg) => {
            player.__ruffle_log__ += msg + "\n";
        };
        player.play();
    }, player);
}

export async function getTraceOutput(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
) {
    // Await any trace output
    await browser.waitUntil(
        async () => {
            return (
                (await browser.execute((player) => {
                    // https://github.com/webdriverio/webdriverio/issues/6486
                    return (player as unknown as Player).__ruffle_log__;
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
        const player = playerElement as unknown as Player;
        const log = player.__ruffle_log__;
        player.__ruffle_log__ = "";
        return log;
    }, player);
}

export async function injectRuffleAndWait(browser: WebdriverIO.Browser) {
    await injectRuffle(browser);
    await waitForRuffle(browser);
}

export async function waitForPlayerToLoad(
    browser: WebdriverIO.Browser,
    player: WebdriverIO.Element,
) {
    await browser.waitUntil(
        async () => await isRufflePlayerLoaded(browser, player),
        {
            timeoutMsg: "Expected Ruffle to load",
            timeout: 60000,
        },
    );
    await throwIfError(browser);
}

export async function openTest(
    browser: WebdriverIO.Browser,
    directory: string,
    filename: string = "index.html",
) {
    await browser.url(`http://localhost:4567/test/${directory}/${filename}`);
}

/** Test set-up for JS API testing. */
export function loadJsAPI(swf?: string) {
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
                    await (player as unknown as Player).load(swf);
                },
                player,
                swf,
            );
            await playAndMonitor(browser, player);
        }
    });
}
