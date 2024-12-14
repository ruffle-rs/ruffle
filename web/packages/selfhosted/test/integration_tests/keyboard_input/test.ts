import { loadJsAPI, getTraceOutput } from "../../utils.js";
import { use, expect } from "chai";
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
        const actualOutput = await getTraceOutput(browser, player);
        expect(actualOutput).to.eql(
            `onKeyDown
event.charCode = 97
event.keyCode = 65

onKeyUp
event.charCode = 97
event.keyCode = 65

`,
        );
    });

    it("enter key is recognised", async () => {
        const player = await browser.$("<ruffle-player>");
        await player.click();

        await browser.keys([Key.Enter]);
        const actualOutput = await getTraceOutput(browser, player);
        expect(actualOutput).to.eql(
            `onKeyDown
event.charCode = 13
event.keyCode = 13

onKeyUp
event.charCode = 13
event.keyCode = 13

`,
        );
    });
});
