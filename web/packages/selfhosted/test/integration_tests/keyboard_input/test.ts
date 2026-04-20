import {
    loadJsAPI,
    expectTraceOutput,
    assertNoMoreTraceOutput,
} from "../../utils.js";
import { use } from "chai";
import chaiHtml from "chai-html";
import { Key } from "webdriverio";

use(chaiHtml);

describe("Key up and down events work", () => {
    loadJsAPI("/test/integration_tests/keyboard_input/test.swf");

    it("'a' key is recognised", async () => {
        const player = await browser.$("<ruffle-player>");
        await player.click();
        // Extra safety click in case there's a modal
        await player.click();

        await browser.keys("a");
        await expectTraceOutput(browser, player, [
            "onKeyDown",
            "event.charCode = 97",
            "event.keyCode = 65",
            "",
            "onKeyUp",
            "event.charCode = 97",
            "event.keyCode = 65",
            "",
        ]);
    });

    it("enter key is recognised", async () => {
        const player = await browser.$("<ruffle-player>");
        await player.click();

        await browser.keys([Key.Enter]);
        await expectTraceOutput(browser, player, [
            "onKeyDown",
            "event.charCode = 13",
            "event.keyCode = 13",
            "",
            "onKeyUp",
            "event.charCode = 13",
            "event.keyCode = 13",
            "",
        ]);
    });

    it("no more traces", async function () {
        const player = await browser.$("<ruffle-object>");
        assertNoMoreTraceOutput(browser, player);
    });
});
