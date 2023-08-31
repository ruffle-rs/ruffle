const { openTest, injectRuffleAndWait } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("Get correct sizes for Flash embeds with percent-based heights", () => {
    it("loads the test", async () => {
        await openTest(browser, __dirname);
    });

    it("is the correct height", async () => {
        await injectRuffleAndWait(browser);
        await browser.$("#result_1").waitForExist();
        await browser.$("#result_2").waitForExist();
        await browser.$("#result_3").waitForExist();
        await browser.$("#result_4").waitForExist();
        await browser.$("#result_5").waitForExist();
        await browser.$("#result_6").waitForExist();
        const actual = await browser.$("#result").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
