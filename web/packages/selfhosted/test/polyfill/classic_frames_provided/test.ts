import { openTest } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Flash inside frame with provided ruffle", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/classic_frames_provided`);
    });

    it("polyfills inside a frame", async () => {
        await browser.switchToFrame(await browser.$("#test-frame"));
        await browser.$("<ruffle-object />").waitForExist();

        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });

    it("polyfills even after a reload", async () => {
        // Contaminate the old contents, to ensure we get a "fresh" state
        await browser.execute(() => {
            document.getElementById("test-container")?.remove();
        });

        // Then reload
        await browser.switchToFrame(null);
        await browser.switchToFrame(await browser.$("#nav-frame"));
        await browser.$("#reload-link").click();

        // And finally, check
        await browser.switchToFrame(null);
        await browser.switchToFrame(await browser.$("#test-frame"));
        await browser.$("<ruffle-object />").waitForExist();

        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });
});
