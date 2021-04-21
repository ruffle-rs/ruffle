const {
    open_test,
    inject_ruffle_and_wait,
    play_and_monitor,
} = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("SWF extension, file with fragment", () => {
    it("loads the test", () => {
        open_test(browser, __dirname);
    });

    it("Polyfills", () => {
        inject_ruffle_and_wait(browser);
        browser.$("<ruffle-object />").waitForExist();

        const actual = browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", () => {
        play_and_monitor(
            browser,
            browser.$("#test-container").$("<ruffle-object />")
        );
    });
});
