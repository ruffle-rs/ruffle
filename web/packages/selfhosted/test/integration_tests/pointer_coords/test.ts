import {
    expectTraceOutput,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";

// The page pins a 200x200 SWF stage at the viewport origin and wraps it in a
// `zoom: 0.5` parent (see index.html), so the canvas's visual rect is the fixed
// box (0, 0, 100, 100) while the backing buffer stays 200x200. Each pointer
// event is dispatched with hardcoded `clientX/Y` aimed at a known visual
// position; the trace output verifies the position translated through to the
// SWF stage uses the visual rect (a visual offset v inside the 100px-wide rect
// maps to stage 2*v across the 200px-wide stage).
//
// `zoom` is deliberate: it is the case the fix exists for. Under `zoom` a
// synthetic event's offsetX is reported in the zoomed/visual space, so the old
// `offsetX * dpr` mapping produces half the correct stage coord and every case
// below fails without the fix. (A `transform: scale` fixture would not catch
// the regression: there offsetX is reported in unscaled layout space and the
// old mapping already happened to be correct.) Every dispatch uses non-zero
// coords so each callback genuinely distinguishes the old mapping from the new.
//
// Coordinates are hardcoded rather than read from getBoundingClientRect so the
// test doesn't lean on the same API the fix relies on.
describe("Pointer coords under CSS zoom", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/pointer_coords");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>").getElement();
        await playAndMonitor(browser, player, ["Loaded!"]);
    });

    it("pointerdown maps client coords through the visual rect", async () => {
        const player = await browser.$("#objectElement").getElement();
        // Visual (50, 50) → stage (100, 100); old mapping would land (50, 50).
        await browser.execute((el) => {
            const canvas = el.shadowRoot!.querySelector(
                "canvas",
            ) as HTMLCanvasElement;
            canvas.dispatchEvent(
                new PointerEvent("pointerdown", {
                    clientX: 50,
                    clientY: 50,
                    button: 0,
                    bubbles: true,
                }),
            );
        }, player);
        await expectTraceOutput(browser, player, [
            "onMouseMove(100,100)",
            "onMouseDown(100,100)",
        ]);
    });

    it("pointerup maps client coords through the visual rect", async () => {
        const player = await browser.$("#objectElement").getElement();
        // Visual (30, 60) → stage (60, 120); old mapping would land (30, 60).
        await browser.execute((el) => {
            const canvas = el.shadowRoot!.querySelector(
                "canvas",
            ) as HTMLCanvasElement;
            canvas.dispatchEvent(
                new PointerEvent("pointerup", {
                    clientX: 30,
                    clientY: 60,
                    button: 0,
                    bubbles: true,
                }),
            );
        }, player);
        await expectTraceOutput(browser, player, [
            "onMouseMove(60,120)",
            "onMouseUp(60,120)",
        ]);
    });

    it("pointermove maps client coords through the visual rect", async () => {
        const player = await browser.$("#objectElement").getElement();
        // Visual (25, 75) → stage (50, 150); old mapping would land (25, 75).
        await browser.execute((el) => {
            const canvas = el.shadowRoot!.querySelector(
                "canvas",
            ) as HTMLCanvasElement;
            canvas.dispatchEvent(
                new PointerEvent("pointermove", {
                    clientX: 25,
                    clientY: 75,
                    bubbles: true,
                }),
            );
        }, player);
        await expectTraceOutput(browser, player, ["onMouseMove(50,150)"]);
    });
});
