import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";
import { Setup } from "ruffle-core";

use(chaiHtml);

declare global {
    interface Window {
        brokenMap: unknown;
        breakMap: () => void;
    }
}

async function expectPolyfilled() {
    await injectRuffleAndWait(browser);
    await browser.$("<ruffle-embed />").waitForExist();
    const actual = await browser
        .$("#test-container")
        .getHTML({ includeSelectorTag: false, pierceShadowRoot: false });
    const expected = fs.readFileSync(
        `${import.meta.dirname}/expected.html`,
        "utf8",
    );
    expect(actual).html.to.equal(expected);
}

async function expectMapUnchanged() {
    const unchanged = await browser.execute(
        () => window.Map === window.brokenMap,
    );
    expect(unchanged).to.equal(true);
}

async function expectMoviePlays() {
    await playAndMonitor(
        browser,
        await browser.$("#test-container").$("<ruffle-embed />").getElement(),
    );
}

describe("Page with a broken Map", () => {
    it("loads the test", async () => {
        await openTest(
            browser,
            `polyfill/broken_map`,
            "index_before_load.html",
        );
    });

    it("polyfills with Ruffle", async () => {
        await expectPolyfilled();
    });

    it("shows translated texts", async () => {
        const text = await browser.execute(
            (player) =>
                player.shadowRoot!.getElementById("acceleration-text")!
                    .textContent,
            await browser
                .$("#test-container")
                .$("<ruffle-embed />")
                .getElement(),
        );
        expect(text).to.not.equal("enable-hardware-acceleration");
        expect(text).to.not.equal("");
    });

    it("doesn't modify the page's Map", async () => {
        await expectMapUnchanged();
    });

    it("plays a movie", async () => {
        await expectMoviePlays();
    });
});

describe("Page breaking Map after Ruffle loads", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/broken_map`, "index_after_load.html");
    });

    it("polyfills with Ruffle", async () => {
        await expectPolyfilled();
    });

    it("shows translated texts in a player created afterwards", async () => {
        const text = await browser.execute(() => {
            window.breakMap();
            const player = (window.RufflePlayer as Setup.PublicAPI)
                .newest()!
                .createPlayer();
            document.body.appendChild(player);
            return player.shadowRoot!.querySelector(
                "#save-manager .modal-button",
            )!.textContent;
        });
        expect(text).to.not.equal("save-backup-all");
        expect(text).to.not.equal("");
    });

    it("doesn't modify the page's Map", async () => {
        await expectMapUnchanged();
    });

    it("plays a movie", async () => {
        await expectMoviePlays();
    });
});
