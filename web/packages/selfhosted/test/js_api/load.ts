import { loadJsAPI, playAndMonitor } from "../utils.js";
import { use } from "chai";
import chaiHtml from "chai-html";
import { RufflePlayer } from "ruffle-core";

use(chaiHtml);

describe("RufflePlayer.load", () => {
    loadJsAPI();

    it("loads and plays a URL", async () => {
        const player = await browser.$("<ruffle-player>");
        await browser.execute((playerElement) => {
            // https://github.com/webdriverio/webdriverio/issues/6486
            const player = playerElement as unknown as RufflePlayer;
            player.load("/test_assets/example.swf");
        }, player);
        await playAndMonitor(browser, player);
    });
});
