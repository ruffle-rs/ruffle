import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Object without type and classid attributes", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/object_missing_type_and_classid`);
    });

    it("polyfills with ruffle", async () => {
        await injectRuffleAndWait(browser);
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
