const { js_api_before, play_and_monitor } = require("../utils");
const { use } = require("chai");
const chaiHtml = require("chai-html");

use(chaiHtml);

describe("RufflePlayer.load", () => {
    js_api_before();

    it("loads and plays a URL", () => {
        const player = browser.$("<ruffle-player>");
        browser.execute((player) => {
            player.load("/test_assets/example.swf");
        }, player);
        play_and_monitor(browser, player);
    });
});
