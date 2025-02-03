import {
    getTraceOutput,
    hideHardwareAccelerationModal,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

async function scroll(
    browser: WebdriverIO.Browser,
    player: ChainablePromiseElement,
    x: number,
    y: number,
    lines: number,
) {
    const canvas = await player.shadow$("canvas");

    return await browser.execute(
        (element, x, y, lines) => {
            const el = element as unknown as HTMLElement;
            el.dispatchEvent(
                new PointerEvent("pointermove", {
                    clientX: x,
                    clientY: y,
                }),
            );
            return el.dispatchEvent(
                new WheelEvent("wheel", {
                    deltaY: lines,
                    deltaMode: WheelEvent.DOM_DELTA_LINE,
                    cancelable: true,
                }),
            );
        },
        canvas,
        x,
        y,
        lines,
    );
}

describe("Mouse Wheel AVM2", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/mouse_wheel_avm2");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded!\n");
        await hideHardwareAccelerationModal(browser, player);
    });

    it("scroll the first clip", async () => {
        const player = await browser.$("#objectElement");

        expect(await scroll(browser, player, 100, 100, 1)).to.equal(false);

        expect(await getTraceOutput(browser, player)).to.equal(
            "Wheel consumed 1, vscroll: 1\n",
        );
    });

    it("scroll the text field", async () => {
        const player = await browser.$("#objectElement");

        expect(await scroll(browser, player, 300, 100, 1)).to.equal(false);
    });

    it("scroll the second clip", async () => {
        const player = await browser.$("#objectElement");

        expect(await scroll(browser, player, 500, 100, 1)).to.equal(false);

        expect(await getTraceOutput(browser, player)).to.equal(
            "Wheel consumed 2, vscroll: 2\n",
        );
    });

    it("scroll non-interactive content", async () => {
        const player = await browser.$("#objectElement");

        expect(await scroll(browser, player, 700, 100, 1)).to.equal(true);
    });
});
