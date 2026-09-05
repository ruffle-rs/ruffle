import { expect } from "chai";
import { Player, Setup } from "ruffle-core";
import { injectRuffleAndWait, playAndMonitor } from "../utils.js";

declare global {
    interface Window {
        lifecycleTest: {
            player: Player.PlayerElement;
            pending: Promise<string>;
            entered: Promise<void>;
            release: (fail?: boolean) => void;
            signal: AbortSignal | null;
        };
    }
}

/** Hold one asynchronous load boundary, including responses that ignore abort. */
async function deferLoad(stage: "wasm" | "font" | "font body") {
    await browser.execute((stage) => {
        const state = window.lifecycleTest;
        let entered!: () => void;
        state.entered = new Promise<void>((resolve) => (entered = resolve));
        const gate = new Promise<boolean | undefined>(
            (resolve) => (state.release = resolve),
        );
        const fetch = window.fetch;
        window.fetch = async (input, init) => {
            const url = String(input);
            if (
                stage === "wasm"
                    ? url.endsWith(".wasm")
                    : url.endsWith("/delayed-font.ttf")
            ) {
                window.fetch = fetch;
                state.signal = init?.signal ?? null;
                if (stage === "font body") {
                    const response = new Response();
                    response.arrayBuffer = async () => {
                        entered();
                        await gate;
                        return new ArrayBuffer(0);
                    };
                    return response;
                }
                entered();
                if (await gate) {
                    throw new Error("Delayed download failed");
                }
                return stage === "wasm" ? fetch(input, init) : new Response();
            }
            return fetch(input, init);
        };
        state.pending = state.player
            .ruffle()
            .load({
                url: "/test_assets/example.swf",
                ...(stage !== "wasm"
                    ? { fontSources: ["/delayed-font.ttf"] }
                    : {}),
            })
            .then(() => "", String);
    }, stage);
    await browser.execute(async () => await window.lifecycleTest.entered);
}

describe("Player lifecycle", () => {
    beforeEach(async () => {
        await browser.url("http://localhost:4567/test_assets/js_api.html");
        await injectRuffleAndWait(browser);
        await browser.execute(() => {
            const player = (window.RufflePlayer as Setup.PublicAPI)
                .newest()!
                .createPlayer();
            player.id = "ruffle-player";
            document.getElementById("test-container")!.appendChild(player);
            window.lifecycleTest = {
                player,
                pending: Promise.resolve(""),
                entered: Promise.resolve(),
                release: () => {},
                signal: null,
            };
        });
    });

    it("releases document listeners, including an open context menu, on every removal", async () => {
        const counts = await browser.execute(() => {
            const active = new Set<EventListenerOrEventListenerObject>();
            const restore: (() => void)[] = [];
            for (const target of [document, document.documentElement]) {
                const add = target.addEventListener;
                const remove = target.removeEventListener;
                target.addEventListener = function (
                    type: string,
                    listener: EventListenerOrEventListenerObject | null,
                    options?: boolean | AddEventListenerOptions,
                ) {
                    if (listener) {
                        active.add(listener);
                        add.call(this, type, listener, options);
                    }
                };
                target.removeEventListener = function (
                    type: string,
                    listener: EventListenerOrEventListenerObject | null,
                    options?: boolean | EventListenerOptions,
                ) {
                    if (listener) {
                        active.delete(listener);
                        remove.call(this, type, listener, options);
                    }
                };
                restore.push(() => {
                    target.addEventListener = add;
                    target.removeEventListener = remove;
                });
            }
            const player = (window.RufflePlayer as Setup.PublicAPI)
                .newest()!
                .createPlayer();
            const counts = [active.size];
            try {
                for (let i = 0; i < 3; i++) {
                    document
                        .getElementById("test-container")!
                        .appendChild(player);
                    counts.push(active.size);
                    player.dispatchEvent(new MouseEvent("contextmenu"));
                    player.remove();
                    counts.push(active.size);
                }
            } finally {
                player.remove();
                restore.forEach((fn) => fn());
            }
            return counts;
        });
        expect(counts).to.deep.equal([0, 2, 0, 2, 0, 2, 0]);
    });

    for (const stage of ["wasm", "font", "font body"] as const) {
        it(`does not resurrect a player removed during ${stage} loading`, async () => {
            await deferLoad(stage);
            const result = await browser.execute(async () => {
                const state = window.lifecycleTest;
                state.player.remove();
                state.release();
                const error = await state.pending;
                return {
                    error,
                    canvases:
                        state.player.shadowRoot!.querySelectorAll("canvas")
                            .length,
                    playing: state.player.isPlaying,
                    readyState: state.player.ruffle().readyState,
                };
            });
            expect(result).to.deep.equal({
                error: "",
                canvases: 0,
                playing: false,
                readyState: 0,
            });
            if (stage !== "wasm") {
                expect(
                    await browser.execute(
                        () => window.lifecycleTest.signal?.aborted,
                    ),
                ).to.equal(true);
            }
        });

        it(`preserves a newer load when an earlier ${stage} load finishes`, async () => {
            await deferLoad(stage);
            await browser.execute(async (stage) => {
                const state = window.lifecycleTest;
                const next = state.player.ruffle().load({
                    url: "/test_assets/example.swf",
                });
                // The WASM download is shared; other work belongs to one load.
                if (stage === "wasm") {
                    state.release();
                }
                await next;
                state.player.ruffle().volume = 0.25;
                state.release();
                const error = await state.pending;
                if (error) {
                    throw new Error(error);
                }
            }, stage);
            await playAndMonitor(
                browser,
                await browser.$("#ruffle-player").getElement(),
            );
            const result = await browser.execute(() => ({
                volume: window.lifecycleTest.player.ruffle().volume,
                canvases:
                    window.lifecycleTest.player.shadowRoot!.querySelectorAll(
                        "canvas",
                    ).length,
            }));
            expect(result).to.deep.equal({ volume: 0.25, canvases: 1 });
        });
    }

    it("ignores an obsolete load failure after reconnection", async () => {
        await deferLoad("font");
        await browser.execute(async () => {
            const state = window.lifecycleTest;
            state.player.remove();
            document
                .getElementById("test-container")!
                .appendChild(state.player);
            await state.player.ruffle().load("/test_assets/example.swf");
            state.release(true);
            const error = await state.pending;
            if (error) {
                throw new Error(error);
            }
        });
        await playAndMonitor(
            browser,
            await browser.$("#ruffle-player").getElement(),
        );
        expect(
            await browser.execute(
                () =>
                    window.lifecycleTest.player.shadowRoot!.querySelectorAll(
                        "canvas",
                    ).length,
            ),
        ).to.equal(1);
    });

    it("cancels removal during the Firefox audio startup delay", async function () {
        if (browser.capabilities.browserName?.toLowerCase() !== "firefox") {
            this.skip();
        }
        const result = await browser.execute(async () => {
            const player = window.lifecycleTest.player;
            const setTimeout = window.setTimeout;
            let entered!: () => void;
            const waiting = new Promise<void>((resolve) => (entered = resolve));
            let release!: () => void;
            window.setTimeout = ((
                handler: TimerHandler,
                timeout?: number,
                ...args: unknown[]
            ) => {
                if (timeout === 200 && typeof handler === "function") {
                    window.setTimeout = setTimeout;
                    release = () => handler(...args);
                    entered();
                    return 0;
                }
                return setTimeout(handler, timeout, ...args);
            }) as typeof window.setTimeout;
            const originalResume = AudioContext.prototype.resume;
            AudioContext.prototype.resume = async () => {};
            try {
                const pending = player
                    .ruffle()
                    .load("/test_assets/example.swf")
                    .then(() => "", String);
                await waiting;
                player.remove();
                release();
                return {
                    error: await pending,
                    canvases:
                        player.shadowRoot!.querySelectorAll("canvas").length,
                    readyState: player.ruffle().readyState,
                };
            } finally {
                window.setTimeout = setTimeout;
                AudioContext.prototype.resume = originalResume;
            }
        });
        expect(result).to.deep.equal({ error: "", canvases: 0, readyState: 0 });
    });

    it("animates the splash screen again after cancelling and reconnecting", async () => {
        for (let attempt = 0; attempt < 2; attempt++) {
            await browser.execute(() => {
                document
                    .getElementById("test-container")!
                    .appendChild(window.lifecycleTest.player);
            });
            await deferLoad("font");
            await browser.waitUntil(
                async () =>
                    await browser.execute(() => {
                        const circle =
                            window.lifecycleTest.player.shadowRoot!.querySelector<SVGCircleElement>(
                                ".spinner",
                            )!;
                        const matrix = circle.getCTM();
                        return matrix !== null && Math.abs(matrix.b) > 0.1;
                    }),
                { timeoutMsg: "Expected the loading spinner to rotate" },
            );
            await browser.execute(async () => {
                const state = window.lifecycleTest;
                state.player.remove();
                state.release();
                const error = await state.pending;
                if (error) {
                    throw new Error(error);
                }
            });
        }
    });

    it("still reloads and plays after reconnection", async () => {
        await browser.execute(async () => {
            const player = window.lifecycleTest.player;
            await player.ruffle().load("/test_assets/example.swf");
            await player.ruffle().reload();
            player.remove();
            document.getElementById("test-container")!.appendChild(player);
            await player.ruffle().reload();
        });
        await playAndMonitor(
            browser,
            await browser.$("#ruffle-player").getElement(),
        );
    });
});
