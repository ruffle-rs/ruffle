import { openTest, injectRuffleAndWait, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

// [NA] Disabled for now as the test can take too long on CI
describe.skip("Doesn't error with cross-origin frames", () => {
    it("Loads the test", async () => {
        await openTest(browser, `polyfill/crossorigin_broken_cors`);
    });

    it("Polyfills with ruffle", async () => {
        await injectRuffleAndWait(browser);
        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", async () => {
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("<ruffle-embed />"),
        );
    });
});
