import {
    assertNoMoreTraceOutput,
    expectTraceOutput,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

async function focusElement(element: ChainablePromiseElement) {
    await browser.execute((el) => {
        el.focus();
    }, element);
}

async function typeText(text: string) {
    for (let i = 0; i < text.length; i++) {
        await browser.keys(`${text.charAt(i)}`);
    }
}

// The idea of the test is to verify whether the site
// can programmatically send events to Ruffle.
// This is used by websites e.g. for implementing custom
// input overlays for mobile.
describe("Programmatic Events", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/programmatic_events");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, ["Loaded!"]);
    });

    // See https://github.com/ruffle-rs/ruffle/issues/6952#issuecomment-1133990189
    it("scenario: programmatic pointerdown on the player", async () => {
        const player = await browser.$("#objectElement");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await browser.execute((el) => {
            el.dispatchEvent(new PointerEvent("pointerdown"));
        }, player);

        await browser.execute((el) => {
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "ArrowRight",
                    code: "ArrowRight",
                    keyCode: 39,
                    bubbles: true,
                }),
            );
        }, player);

        await expectTraceOutput(browser, player, ["onKeyDown(0,39)"]);

        await browser.keys("x");

        await expectTraceOutput(browser, player, [
            "onKeyDown(120,88)",
            "onKeyUp(120,88)",
        ]);
    });

    // That has been possible since https://github.com/ruffle-rs/ruffle/pull/17158,
    // so ideally we want to preserve this behavior.
    it("scenario: programmatic focus on the container", async () => {
        const player = await browser.$("#objectElement");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await focusElement(await player.shadow$("#container"));

        await browser.execute((el) => {
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "ArrowRight",
                    code: "ArrowRight",
                    keyCode: 39,
                    bubbles: true,
                }),
            );
        }, player);

        await expectTraceOutput(browser, player, ["onKeyDown(0,39)"]);

        await browser.keys("x");

        await expectTraceOutput(browser, player, [
            "onKeyDown(120,88)",
            "onKeyUp(120,88)",
        ]);
    });

    // That's probably the most sane way of focusing Ruffle.
    it("scenario: programmatic focus on the player", async () => {
        const player = await browser.$("#objectElement");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await focusElement(player);

        await browser.execute((el) => {
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "ArrowRight",
                    code: "ArrowRight",
                    keyCode: 39,
                    bubbles: true,
                }),
            );
        }, player);

        await expectTraceOutput(browser, player, ["onKeyDown(0,39)"]);

        await browser.keys("x");

        await expectTraceOutput(browser, player, [
            "onKeyDown(120,88)",
            "onKeyUp(120,88)",
        ]);
    });

    it("scenario: example input overlay", async () => {
        const player = await browser.$("#objectElement");
        const overlay = await browser.$("#overlay");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await overlay.click();

        await expectTraceOutput(browser, player, ["onKeyDown(0,39)"]);

        await browser.keys("x");

        await expectTraceOutput(browser, player, [
            "onKeyDown(120,88)",
            "onKeyUp(120,88)",
        ]);
    });

    it("no more traces", async function () {
        const player = await browser.$("#objectElement");
        assertNoMoreTraceOutput(browser, player);
    });
});
