const { open_test, inject_ruffle_and_wait } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("iframe onload", () => {
    it("loads the test", async () => {
        await open_test(browser, __dirname);
    });

    it("runs the iframe onload event", async () => {
        await inject_ruffle_and_wait(browser);
        await browser.$("<div />").waitForExist();

        const actual = await browser.$("#container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
