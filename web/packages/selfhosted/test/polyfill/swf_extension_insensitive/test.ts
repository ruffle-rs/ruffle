import { openTest, injectRuffleAndWait, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("SWF extension insensitive", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/swf_extension_insensitive`);
    });

    it("Polyfills", async () => {
        await injectRuffleAndWait(browser);
        await browser.$("<ruffle-object />").waitForExist();

        const actual = await browser
            .$("#test-container")
            .getHTML({ includeSelectorTag: false, pierceShadowRoot: false });
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", async () => {
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("<ruffle-object />"),
        );
    });
});
