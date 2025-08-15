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
                    clientX: x as unknown as number,
                    clientY: y as unknown as number,
                }),
            );
            return el.dispatchEvent(
                new WheelEvent("wheel", {
                    deltaY: lines as unknown as number,
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
            await playAndMonitor(browser, player, "Loaded!\n");
            await hideHardwareAccelerationModal(browser, player);
            // await new Promise(f => setTimeout(f, 10000000));
        });

        it("scroll the first clip", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 100, 100, 1)).to.equal(
                expectedScroll ?? false,
            );

            expect(await getTraceOutput(browser, player)).to.equal(
                "Wheel consumed 1, vscroll: 1\n",
            );
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

            expect(await getTraceOutput(browser, player)).to.equal(
                "Wheel consumed 2, vscroll: 2\n",
            );
        });

        it("scroll non-interactive content", async () => {
            const player = await browser.$("#objectElement");

            expect(await scroll(browser, player, 700, 100, 1)).to.equal(
                expectedScroll ?? true,
            );
        });
    });
});
