const { jsApiBefore } = require("../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");

use(chaiHtml);

describe("RufflePlayer.metadata", () => {
    jsApiBefore("/test_assets/example.swf");

    it("has metadata after load", async () => {
        const player = await browser.$("<ruffle-player>");
        const metadata = await browser.execute(
            (player) => player.metadata,
            player,
        );
        expect(metadata).to.eql({
            width: 550,
            height: 400,
            frameRate: 24,
            numFrames: 1,
            swfVersion: 15,
            isActionScript3: false,
            backgroundColor: "#FF0000",
            uncompressedLength: 1450,
        });
    });
});
