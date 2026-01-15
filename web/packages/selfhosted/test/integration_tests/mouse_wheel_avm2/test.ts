import {
    assertNoMoreTraceOutput,
    expectTraceOutput,
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
    const canvasSize = await canvas.getSize();

    const xOffset = x - canvasSize.width / 2;
    const yOffset = y - canvasSize.height / 2;
    await canvas.moveTo({ xOffset, yOffset });

    return await browser.execute(
        (element, lines) => {
            const el = element as unknown as HTMLElement;
            return el.dispatchEvent(
                new WheelEvent("wheel", {
                    deltaY: lines as unknown as number,
                    deltaMode: WheelEvent.DOM_DELTA_LINE,
                    cancelable: true,
                }),
            );
        },
        canvas,
        lines,
    );
}

interface TestParams {
    name: string;
    expectedScroll: boolean | null;
}

[
    {
        name: "always",
        expectedScroll: true,
    },
    {
        name: "never",
        expectedScroll: false,
    },
    {
        name: "smart",
        expectedScroll: null,
    },
    {
        name: "default",
        expectedScroll: null,
    },
].forEach(function (testParams: TestParams) {
    describe("Mouse Wheel AVM2, behavior: " + testParams.name, () => {
        const expectedScroll = testParams.expectedScroll;

        it("load the test", async () => {
            await openTest(
                browser,
                "integration_tests/mouse_wheel_avm2",
                "index_" + testParams.name + ".html",
            );
            await injectRuffleAndWait(browser);
            const player = await browser.$("<ruffle-object>");
            await playAndMonitor(browser, player, ["Loaded!"]);
            await hideHardwareAccelerationModal(browser, player);
            // await new Promise(f => setTimeout(f, 10000000));
        });

        it("scroll the first clip", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 100, 100, 1)).to.equal(
                expectedScroll ?? false,
            );
            await expectTraceOutput(browser, player, [
                "Wheel consumed 1, vscroll: 1",
            ]);
        });

        it("scroll the text field up", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 300, 100, -1)).to.equal(
                expectedScroll ?? true,
            );
        });

        it("scroll the text field", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 300, 100, 2)).to.equal(
                expectedScroll ?? false,
            );
        });

        it("scroll the text field back up", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 300, 100, -1)).to.equal(
                expectedScroll ?? false,
            );
        });

        it("scroll the second clip", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 500, 100, 1)).to.equal(
                expectedScroll ?? false,
            );
            await expectTraceOutput(browser, player, [
                "Wheel consumed 2, vscroll: 2",
            ]);
        });

        it("scroll non-interactive content", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 700, 100, 1)).to.equal(
                expectedScroll ?? true,
            );
        });

        it("no more traces", async function () {
            const player = await browser.$("#objectElement");
            assertNoMoreTraceOutput(browser, player);
        });
    });
});
