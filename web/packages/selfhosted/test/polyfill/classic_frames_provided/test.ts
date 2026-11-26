import { openTest, getFrame, getInnerHTML } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Flash inside frame with provided ruffle", () => {
    let page: WebdriverIO.BrowsingContext;

    it("loads the test", async () => {
        page = await openTest(browser, `polyfill/classic_frames_provided`);
    });

    it("polyfills inside a frame", async () => {
        const frame = await getFrame(browser, page, "#test-frame");
        await frame.$("<ruffle-object />").waitForExist();

        const actual = await getInnerHTML(frame.$("#test-container"));
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });

    it("polyfills even after a reload", async () => {
        // Contaminate the old contents, to ensure we get a "fresh" state
        let frame = await getFrame(browser, page, "#test-frame");
        await frame.execute(() => {
            document.getElementById("test-container")?.remove();
        });

        // Then reload
        const nav = await getFrame(browser, page, "#nav-frame");
        await nav.$("#reload-link").click();

        // And finally, check
        frame = await getFrame(browser, page, "#test-frame");
        await frame.$("<ruffle-object />").waitForExist();

        const actual = await getInnerHTML(frame.$("#test-container"));
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });
});
