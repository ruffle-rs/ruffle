const { jsApiBefore, playAndMonitor } = require("../utils");
const { use } = require("chai");
const chaiHtml = require("chai-html");

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
