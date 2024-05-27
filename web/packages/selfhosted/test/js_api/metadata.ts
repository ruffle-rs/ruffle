import { loadJsAPI } from "../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import { RufflePlayer } from "ruffle-core";

use(chaiHtml);

describe("RufflePlayer.metadata", () => {
    loadJsAPI("/test_assets/example.swf");

    it("has metadata after load", async () => {
        const player = await browser.$("<ruffle-player>");
        const metadata = await browser.execute(
            // https://github.com/webdriverio/webdriverio/issues/6486
            (player) => (player as unknown as RufflePlayer).metadata,
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
