import { jsApiBefore, playAndMonitor } from "../utils.js";
import { use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

describe("RufflePlayer.load", () => {
    jsApiBefore();

    it("loads and plays a URL", async () => {
        const player = await browser.$("<ruffle-player>");
        await browser.execute((player) => {
            player.load("/test_assets/example.swf");
        }, player);
        await playAndMonitor(browser, player);
    });
});
