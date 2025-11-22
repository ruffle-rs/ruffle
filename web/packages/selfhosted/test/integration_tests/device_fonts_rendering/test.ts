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

describe("Device Fonts: Rendering", () => {
    for (const deviceFontRenderer of deviceFontRenderers) {
        it(`load the test: ${deviceFontRenderer}`, async () => {
            await openTest(
                browser,
                "integration_tests/device_fonts_rendering",
                `index.html?deviceFontRenderer=${deviceFontRenderer}`,
            );
            await injectRuffleAndWait(browser);
            const player = await browser.$("<ruffle-object>");
            await playAndMonitor(browser, player, ["Loaded test!"]);
            await hideHardwareAccelerationModal(browser, player);
        });

        it("check rendered image", async () => {
            const player = await browser.$("#objectElement");

            await expectTraceOutput(browser, player, ["Character bounds:"]);

            const messages = await getTraceOutput(browser, player, 4);

            const boundsX = Number(messages[0]);
            const boundsWidth = Number(messages[1]);
            const boundsY = Number(messages[2]);
            const boundsHeight = Number(messages[3]);

            expect(boundsX).to.be.greaterThan(0);
            expect(boundsWidth).to.be.greaterThan(0);
            expect(boundsY).to.greaterThan(0);
            expect(boundsHeight).to.be.greaterThan(0);

            const canvas = await player.shadow$("canvas");

            const [canvasWidth, canvasHeight, pixels] = await browser.execute(
                (el) => {
                    const canvas = el as HTMLCanvasElement;
                    const ctx =
                        canvas.getContext("webgl") ||
                        canvas.getContext("webgl2");
                    const pixels = new Uint8Array(
                        canvas.width * canvas.height * 4,
                    );
                    ctx?.readPixels(
                        0,
                        0,
                        canvas.width,
                        canvas.height,
                        ctx.RGBA,
                        ctx.UNSIGNED_BYTE,
                        pixels,
                    );

                    return [canvas.width, canvas.height, [...pixels]];
                },
                canvas,
            );

            let insideBoundsBg = 0;
            let insideBoundsNonBg = 0;

            for (let i = 0; i < pixels.length; i += 4) {
                const pixelIndex = i / 4;
                const pixel = pixels.slice(i, i + 4);
                const x = pixelIndex % canvasWidth;
                const y = Math.floor(pixelIndex / canvasWidth);

                const scaledX = (x * 100) / canvasWidth;
                const scaledY = (y * 100) / canvasHeight;

                const insideBounds =
                    boundsX <= scaledX &&
                    scaledX <= boundsX + boundsWidth &&
                    boundsY <= scaledY &&
                    scaledY <= boundsY + boundsHeight;

                if (insideBounds) {
                    // Background is magenta.
                    const isBackground =
                        pixel.toString() === [255, 0, 255, 255].toString();
                    if (isBackground) {
                        insideBoundsBg += 1;
                    } else {
                        insideBoundsNonBg += 1;
                    }
                } else {
                    // Pixels outside of bounds should be background only.
                    expect(pixel.toString()).to.be.equal(
                        [255, 0, 255, 255].toString(),
                    );
                }
            }

            // Make sure there are both background and black pixels inside bounds.
            expect(insideBoundsBg).is.greaterThan(0);
            expect(insideBoundsNonBg).is.greaterThan(0);
        });

        it("no more traces", async function () {
            const player = await browser.$("#objectElement");
            assertNoMoreTraceOutput(browser, player);
        });
    }
});
