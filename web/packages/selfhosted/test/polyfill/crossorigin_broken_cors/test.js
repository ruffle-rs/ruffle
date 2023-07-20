const {
    openTest,
    injectRuffleAndWait,
    playAndMonitor,
} = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

// [NA] Disabled for now as the test can take too long on CI
describe.skip("Doesn't error with cross-origin frames", () => {
    it("Loads the test", async () => {
        await openTest(browser, __dirname);
    });

    it("Polyfills with ruffle", async () => {
        await injectRuffleAndWait(browser);
        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", async () => {
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("<ruffle-embed />"),
        );
    });
});
