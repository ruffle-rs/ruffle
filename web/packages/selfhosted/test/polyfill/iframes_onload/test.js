const { openTest, injectRuffleAndWait } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("iframe onload", () => {
    it("loads the test", async () => {
        await openTest(browser, __dirname);
    });

    it("runs the iframe onload event", async () => {
        await injectRuffleAndWait(browser);
        await browser.$("<div />").waitForExist();

        const actual = await browser.$("#container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
