import { openTest, injectRuffleAndWait } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("iframe onload", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/iframes_onload`);
    });

    it("runs the iframe onload event", async () => {
        await injectRuffleAndWait(browser);
        await browser.$("<div />").waitForExist();

        const actual = await browser
            .$("#container")
            .getHTML({ includeSelectorTag: false, pierceShadowRoot: false });
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });
});
