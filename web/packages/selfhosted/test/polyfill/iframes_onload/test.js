const { open_test, inject_ruffle_and_wait } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("iframe onload", () => {
    it("loads the test", () => {
        open_test(browser, __dirname);
    });

    it("runs the iframe onload event", () => {
        inject_ruffle_and_wait(browser);
        browser.$("<div />").waitForExist();

        const actual = browser.$("#container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
