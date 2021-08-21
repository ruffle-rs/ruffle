const {
    open_test,
    inject_ruffle_and_wait,
    play_and_monitor,
} = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("SWF extension insensitive", () => {
    it("loads the test", async () => {
        await open_test(browser, __dirname);
    });

    it("Polyfills", async () => {
        await inject_ruffle_and_wait(browser);
        await browser.$("<ruffle-object />").waitForExist();

        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", async () => {
        await play_and_monitor(
            browser,
            await browser.$("#test-container").$("<ruffle-object />")
        );
    });
});
