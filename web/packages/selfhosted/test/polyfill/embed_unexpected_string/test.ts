import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

describe("Embed with unexpected string", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/embed_unexpected_string`);
    });

    it("polyfills with ruffle", async () => {
        await injectRuffleAndWait(browser);
        await browser.$("<ruffle-embed />").waitForExist();
    });

    it("Plays a movie", async () => {
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("<ruffle-embed />"),
        );
    });
});
