/* eslint-disable @typescript-eslint/no-unused-expressions */

import {
    assertNoMoreTraceOutput,
    expectTraceOutput,
    getTraceOutput,
    hideHardwareAccelerationModal,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

const deviceFontRenderers = ["embedded", "canvas"];

describe("Device Fonts: Metrics", () => {
    for (const deviceFontRenderer of deviceFontRenderers) {
        it(`load the test: ${deviceFontRenderer}`, async () => {
            await openTest(
                browser,
                "integration_tests/device_fonts_metrics",
                `index.html?deviceFontRenderer=${deviceFontRenderer}`,
            );
            await injectRuffleAndWait(browser);
            const player = await browser.$("<ruffle-object>");
            await playAndMonitor(browser, player, ["Loaded test!"]);
            await hideHardwareAccelerationModal(browser, player);
        });

        it("check metrics", async () => {
            const player = await browser.$("#objectElement");

            await expectTraceOutput(browser, player, [
                "Displayed text metrics:",
            ]);

            const messages = await getTraceOutput(browser, player, 10);

            const textWidth = Number(messages[0]);
            const textHeight = Number(messages[1]);
            const char0x = Number(messages[2]) - 2;
            const char0w = Number(messages[3]);
            const char1x = Number(messages[4]) - char0x;
            const char1w = Number(messages[5]);
            const char2x = Number(messages[6]) - char1x;
            const char2w = Number(messages[7]);
            const char3x = Number(messages[8]) - char2x;
            const char3w = Number(messages[9]);

            expect(textWidth).to.be.greaterThan(0);
            expect(textHeight).to.be.greaterThan(0);
            expect(char0x).to.equal(0);
            expect(char0w).to.be.greaterThan(0);
            expect(char1x).to.be.greaterThan(0);
            expect(char1w).to.be.greaterThan(0);
            expect(char2x).to.be.greaterThan(0);
            expect(char2w).to.be.greaterThan(0);

            if (deviceFontRenderer === "embedded") {
                // Embedded renderer does not support CJK characters by default
                expect(char3x).to.be.NaN;
                expect(char3w).to.be.NaN;
            } else {
                expect(char3x).to.be.greaterThan(0);
                expect(char3w).to.be.greaterThan(0);
            }
        });

        it("no more traces", async function () {
            const player = await browser.$("#objectElement");
            assertNoMoreTraceOutput(browser, player);
        });
    }
});
