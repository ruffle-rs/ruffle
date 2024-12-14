import { loadJsAPI } from "../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

describe("Exposed RufflePlayer methods/properties", () => {
    loadJsAPI();

    it("exposed API has not changed", async () => {
        const player = await browser.$("<ruffle-player>");
        const keys = await browser.execute(async (playerElement) => {
            // https://github.com/webdriverio/webdriverio/issues/6486
            const player = playerElement as unknown;
            return Reflect.ownKeys(Object.getPrototypeOf(player));
        }, player);
        expect(keys).to.have.members([
            // FlashAPI
            "PercentLoaded",
            // LegacyRuffleAPI
            "onFSCommand",
            "config",
            "loadedConfig",
            "readyState",
            "metadata",
            "reload",
            "load",
            "play",
            "isPlaying",
            "volume",
            "fullscreenEnabled",
            "isFullscreen",
            "setFullscreen",
            "enterFullscreen",
            "exitFullscreen",
            "pause",
            "traceObserver",
            "downloadSwf",
            "displayMessage",
            // PlayerElement
            "ruffle",
            // RufflePlayerElement
            "attributeChangedCallback",
            "connectedCallback",
            "constructor",
            "debugPlayerInfo",
            "disconnectedCallback",
        ]);
    });
});
