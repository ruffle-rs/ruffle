const { inject_ruffle_and_wait, open_test } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("Object with ruffle-embed tag", () => {
    it("loads the test", () => {
        open_test(browser, __dirname);
    });

    it("already polyfilled with ruffle", () => {
        inject_ruffle_and_wait(browser);
        const actual = browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
