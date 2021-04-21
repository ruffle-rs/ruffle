const {
    inject_ruffle_and_wait,
    open_test,
    play_and_monitor,
} = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("Object tag", () => {
    it("loads the test", () => {
        open_test(browser, __dirname);
    });

    it("polyfills with ruffle", () => {
        inject_ruffle_and_wait(browser);
        const actual = browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", () => {
        play_and_monitor(
            browser,
            browser.$("#test-container").$("<ruffle-embed />")
        );
    });
});
