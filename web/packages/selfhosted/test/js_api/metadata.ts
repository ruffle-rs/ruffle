import { loadJsAPI } from "../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import { Player } from "ruffle-core";

use(chaiHtml);

describe("RufflePlayer.metadata", () => {
    loadJsAPI("/test_assets/example.swf");

    it("has metadata after load", async () => {
        const player = await browser.$("<ruffle-player>");
        const metadata = await browser.execute(
            // https://github.com/webdriverio/webdriverio/issues/6486
            (player) => (player as unknown as Player).metadata,
            player,
        );
        // [NA] Work around a chrome 87 bug where it's (somehow) adding extra data to this object
        if (metadata && "capabilities" in metadata) {
            delete metadata.capabilities;
        }
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
