const { injectRuffleAndWait, openTest } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("PDF with .swf GET", () => {
    it("loads the test", async () => {
        await openTest(browser, __dirname);
    });

    it("doesn't polyfill with ruffle", async () => {
        await injectRuffleAndWait(browser);
        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
