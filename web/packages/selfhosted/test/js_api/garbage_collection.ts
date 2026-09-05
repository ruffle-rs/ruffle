import { expect } from "chai";
import { Player, Setup } from "ruffle-core";
import { injectRuffleAndWait } from "../utils.js";

declare global {
    interface Window {
        memoryTest: {
            player: Player.PlayerElement;
            removed: WeakRef<Player.PlayerElement>[];
        };
    }
}

/** Run GC and cycle collection in the disposable Firefox test profile. */
async function collectGarbage() {
    await browser.setMozContext("chrome");
    try {
        // Use the classic WebDriver command: BiDi execution ignores moz/context.
        await browser.executeScript(
            `return new Promise(resolve => {
                Cc["@mozilla.org/memory-reporter-manager;1"]
                    .getService(Ci.nsIMemoryReporterManager)
                    .minimizeMemoryUsage(() => resolve());
            });`,
            [],
        );
    } finally {
        await browser.setMozContext("content");
    }
}

describe("Player garbage collection", () => {
    beforeEach(async function () {
        if (
            process.env["RUFFLE_TEST_GC"] !== "1" ||
            !browser.isFirefox ||
            process.argv.includes("--browserstack")
        ) {
            this.skip();
        }
        await browser.url("http://localhost:4567/test_assets/js_api.html");
        await injectRuffleAndWait(browser);
        await browser.execute(async () => {
            const player = (window.RufflePlayer as Setup.PublicAPI)
                .newest()!
                .createPlayer();
            document.getElementById("test-container")!.appendChild(player);
            window.memoryTest = { player, removed: [] };
            await player.ruffle().load("/test_assets/example.swf");
        });
    });

    for (const waitForMetadata of [false, true]) {
        it(`collects removed players ${waitForMetadata ? "after metadata" : "during SWF loading"}, with cold and cached WASM`, async () => {
            for (let batch = 0; batch < 3; batch++) {
                await browser.execute(async (waitForMetadata) => {
                    const state = window.memoryTest;
                    for (let i = 0; i < 20; i++) {
                        state.removed.push(new WeakRef(state.player));
                        state.player.remove();
                        state.player = (window.RufflePlayer as Setup.PublicAPI)
                            .newest()!
                            .createPlayer();
                        document
                            .getElementById("test-container")!
                            .appendChild(state.player);
                        const loaded = waitForMetadata
                            ? new Promise<void>((resolve) =>
                                  state.player.addEventListener(
                                      "loadeddata",
                                      () => resolve(),
                                      { once: true },
                                  ),
                              )
                            : Promise.resolve();
                        await state.player
                            .ruffle()
                            .load("/test_assets/example.swf");
                        await loaded;
                    }
                }, waitForMetadata);
                await collectGarbage();
                const retained = await browser.execute(() =>
                    window.memoryTest.removed.flatMap((ref, index) =>
                        ref.deref() ? [index] : [],
                    ),
                );
                expect(
                    retained,
                    `retained players after batch ${batch}`,
                ).to.deep.equal([]);
            }
        });
    }
});
