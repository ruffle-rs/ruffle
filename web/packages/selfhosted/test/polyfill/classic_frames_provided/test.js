const { open_test } = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

describe("Flash inside frame with provided ruffle", () => {
    it("loads the test", () => {
        open_test(browser, __dirname);
    });

    it("polyfills inside a frame", () => {
        browser.switchToFrame(browser.$("#test-frame"));
        browser.$("<ruffle-object />").waitForExist();

        const actual = browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });

    it("polyfills even after a reload", () => {
        // Contaminate the old contents, to ensure we get a "fresh" state
        browser.execute(() => {
            document.getElementById("test-container").remove();
        });

        // Then reload
        browser.switchToParentFrame();
        browser.switchToFrame(browser.$("#nav-frame"));
        browser.$("#reload-link").click();

        // And finally, check
        browser.switchToParentFrame();
        browser.switchToFrame(browser.$("#test-frame"));
        browser.$("<ruffle-object />").waitForExist();

        const actual = browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });
});
