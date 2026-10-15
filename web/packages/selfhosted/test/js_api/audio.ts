import { expect } from "chai";
import { Player, Setup } from "ruffle-core";
import { injectRuffleAndWait, loadJsAPI } from "../utils.js";

describe("Audio buffer cleanup", () => {
    loadJsAPI();

    // wasm-bindgen caches imported constructors for the document's lifetime.
    beforeEach(async () => {
        await browser.refresh();
        await injectRuffleAndWait(browser);
    });

    for (const reload of [false, true]) {
        it(`releases audio callbacks when a player is ${reload ? "reloaded" : "removed"}`, async () => {
            const result = await browser.execute(async (reload) => {
                type Context = {
                    audio: AudioContext;
                    sources: AudioBufferSourceNode[];
                    closed: Promise<void> | null;
                };
                const contexts: Context[] = [];
                const AudioContext = window.AudioContext;
                window.AudioContext = new Proxy(AudioContext, {
                    construct(target, args) {
                        const audio = Reflect.construct(
                            target,
                            args,
                        ) as AudioContext;
                        const record: Context = {
                            audio,
                            sources: [],
                            closed: null,
                        };
                        contexts.push(record);
                        const create = audio.createBufferSource.bind(audio);
                        audio.createBufferSource = () => {
                            const source = create();
                            record.sources.push(source);
                            return source;
                        };
                        const close = audio.close.bind(audio);
                        audio.close = () => {
                            record.closed = close();
                            return record.closed;
                        };
                        return audio;
                    },
                });

                const ruffle = (
                    window.RufflePlayer as Setup.PublicAPI
                ).newest()!;
                const player = ruffle.createPlayer();
                document.getElementById("test-container")!.appendChild(player);
                const load = async () => {
                    await (player as Player.PlayerElement)
                        .ruffle()
                        .load("/test_assets/example.swf");
                    const deadline = performance.now() + 30_000;
                    while (player.ruffle().readyState !== 2) {
                        if (performance.now() > deadline) {
                            throw new Error(
                                "Expected the SWF to finish loading",
                            );
                        }
                        await new Promise((resolve) => setTimeout(resolve, 10));
                    }
                };
                const snapshot = () =>
                    contexts.map(({ audio, sources }) => ({
                        sources: sources.length,
                        cleared: sources.filter(
                            (source) => source.onended === null,
                        ).length,
                        closed: audio.state === "closed",
                    }));
                try {
                    await load();
                    if (reload) {
                        await load();
                    }
                    const before = snapshot();
                    player.remove();
                    await Promise.all(contexts.map(({ closed }) => closed));
                    return { before, after: snapshot() };
                } finally {
                    player.remove();
                    window.AudioContext = AudioContext;
                }
            }, reload);

            expect(result.before).to.have.length(reload ? 2 : 1);
            const active = result.before.at(-1)!;
            expect(active.sources).to.be.at.least(2);
            expect(active.cleared).to.equal(0);
            expect(active.closed).to.equal(false);
            if (reload) {
                expect(result.before[0]!.cleared).to.equal(2);
            }
            for (const context of result.after) {
                expect(context.sources).to.be.at.least(2);
                expect(context.cleared).to.equal(2);
                expect(context.closed).to.equal(true);
            }
        });
    }
});
