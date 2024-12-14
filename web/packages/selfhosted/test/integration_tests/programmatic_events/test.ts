import {
    getTraceOutput,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

async function focusElement(element: ChainablePromiseElement) {
    await browser.execute((element) => {
        const el = element as unknown as HTMLElement;
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
        await playAndMonitor(browser, player, "Loaded!\n");
    });

    // See https://github.com/ruffle-rs/ruffle/issues/6952#issuecomment-1133990189
    it("scenario: programmatic pointerdown on the player", async () => {
        const player = await browser.$("#objectElement");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.dispatchEvent(new PointerEvent("pointerdown"));
        }, player);

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "ArrowRight",
                    code: "ArrowRight",
                    keyCode: 39,
                    bubbles: true,
                }),
            );
        }, player);

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(0,39)\n",
        );

        await browser.keys("x");

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(120,88)\nonKeyUp(120,88)\n",
        );
    });

    // That has been possible since https://github.com/ruffle-rs/ruffle/pull/17158,
    // so ideally we want to preserve this behavior.
    it("scenario: programmatic focus on the container", async () => {
        const player = await browser.$("#objectElement");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await focusElement(await player.shadow$("#container"));

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "ArrowRight",
                    code: "ArrowRight",
                    keyCode: 39,
                    bubbles: true,
                }),
            );
        }, player);

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(0,39)\n",
        );

        await browser.keys("x");

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(120,88)\nonKeyUp(120,88)\n",
        );
    });

    // That's probably the most sane way of focusing Ruffle.
    it("scenario: programmatic focus on the player", async () => {
        const player = await browser.$("#objectElement");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await focusElement(player);

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "ArrowRight",
                    code: "ArrowRight",
                    keyCode: 39,
                    bubbles: true,
                }),
            );
        }, player);

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(0,39)\n",
        );

        await browser.keys("x");

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(120,88)\nonKeyUp(120,88)\n",
        );
    });

    it("scenario: example input overlay", async () => {
        const player = await browser.$("#objectElement");
        const overlay = await browser.$("#overlay");

        await focusElement(await browser.$("#inputElement"));

        await typeText("should be ignored");

        await overlay.click();

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(0,39)\n",
        );

        await browser.keys("x");

        expect(await getTraceOutput(browser, player)).to.equal(
            "onKeyDown(120,88)\nonKeyUp(120,88)\n",
        );
    });
});
