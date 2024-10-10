import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Object tag", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/object_flashvars_in_url`);
    });

    it("polyfills with ruffle", async () => {
        await injectRuffleAndWait(browser);
        const actual = await browser
            .$("#test-container")
            .getHTML({ includeSelectorTag: false, pierceShadowRoot: false });
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie with flashvars", async () => {
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("<ruffle-embed />"),
            `// _level0.a
1

// typeof(a)
string

// _level0.b
3 %3

// typeof(b)
string

// _level0.c


// typeof(c)
string

// _level0.d
undefined

// typeof(d)
undefined

`,
        );
    });
});
