import { openTest, injectRuffleAndWait } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Object without data attribute", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/object_missing_data`);
    });

    it("doesn't polyfill with ruffle", async () => {
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
});
