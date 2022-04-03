const { open_test, inject_ruffle_and_wait } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("Object with Flash YouTube video", () => {
    it("loads the test", async () => {
        await open_test(browser, __dirname);
    });

    it("doesn't polyfill with ruffle", async () => {
        await inject_ruffle_and_wait(browser);
        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
