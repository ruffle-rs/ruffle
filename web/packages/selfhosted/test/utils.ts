import { expect } from "chai";
import { Player, Setup } from "ruffle-core";

declare global {
    interface Window {
        ruffleErrors: ErrorEvent[];
    }
}

declare module "ruffle-core/dist/public/player" {
    interface PlayerElement {
        __ruffle_log__: string[];
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
    player: ChainablePromiseElement,
) {
    return (
        (await browser.execute(
            (player) =>
                // TODO: How can we import ReadyState enum?
                (player as Player.PlayerElement).ruffle().readyState,
            await player,
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
    player: ChainablePromiseElement,
    expectedOutput: string[] = ["Hello from Flash!"],
) {
    await throwIfError(browser);
    await waitForPlayerToLoad(browser, await player);
    await setupAndPlay(browser, await player);

    await expectTraceOutput(browser, player, expectedOutput);
}

export async function setupAndPlay(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
) {
    await browser.execute(
        (playerElement) => {
            const player = playerElement as Player.PlayerElement;
            player.__ruffle_log__ = [];
            player.ruffle().traceObserver = (msg) => {
                player.__ruffle_log__.push(msg);
                console.log(`[trace] ${msg}`);
            };
            player.ruffle().resume();
        },
        await player,
    );
}

export async function getTraceOutput(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    messageCount: number = 1,
): Promise<string[]> {
    // Await trace output
    await browser.waitUntil(
        async () => {
            const log = await browser.execute(
                (player) => {
                    return (player as Player.PlayerElement).__ruffle_log__;
                },
                await player,
            );
            return log.length >= messageCount;
        },
        {
            timeoutMsg: `Expected Ruffle to trace ${messageCount} messages`,
        },
    );

    // Get the output
    return await browser.execute(
        (playerElement, messageCount) => {
            const player = playerElement as Player.PlayerElement;
            const m = messageCount as unknown as number;
            return player.__ruffle_log__.splice(0, m);
        },
        player,
        messageCount,
    );
}

export async function expectTraceOutput(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    messages: string[],
) {
    expect(
        await getTraceOutput(browser, player, messages.length),
    ).to.deep.equal(messages);
}

export async function clearTraceOutput(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
) {
    await browser.execute((playerElement) => {
        const player = playerElement as Player.PlayerElement;
        player.__ruffle_log__ = [];
    }, player);
}

export async function assertNoMoreTraceOutput(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
) {
    await browser.execute((playerElement) => {
        const player = playerElement as Player.PlayerElement;
        if (player.__ruffle_log__.length > 0) {
            const log = player.__ruffle_log__.join("\n");
            throw new Error(`Unexpected trace:\n${log}`);
        }
    }, player);
}

export async function injectRuffleAndWait(browser: WebdriverIO.Browser) {
    await injectRuffle(browser);
    await waitForRuffle(browser);
}

export async function waitForPlayerToLoad(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
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

        player = await browser.execute(() => {
            const ruffle = (window.RufflePlayer as Setup.PublicAPI).newest();
            const player = ruffle!.createPlayer();
            const container = document.getElementById("test-container");
            container!.appendChild(player);
            return player;
        });

        if (swf) {
            await browser.execute(
                async (player, swf) => {
                    await (player as Player.PlayerElement).ruffle().load(swf);
                },
                player,
                swf,
            );
            await playAndMonitor(browser, player);
        }
    });
}

export async function closeAllModals(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
) {
    const modals = await player.$$(".modal:not(.hidden)");
    await browser.execute(
        ({ modals }) => {
            for (const m of modals) {
                const cl = m.querySelector(".close-modal")! as HTMLElement;
                cl.click();
            }
        },
        { modals },
    );
}

export async function hideHardwareAccelerationModal(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
) {
    // Trigger it if not triggered yet
    await player.moveTo();
    await closeAllModals(browser, player);
}
