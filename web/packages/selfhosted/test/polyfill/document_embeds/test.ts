import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { use, expect } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Document embeds", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/document_embeds`);
    });

    it("Accesses the right number of elements with ruffle", async () => {
        await injectRuffleAndWait(browser);
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("ruffle-embed#emb1"),
        );
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("ruffle-embed#emb2"),
        );
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("ruffle-embed#emb3"),
        );
        await browser.execute(() => {
            function countDocumentEmbeds() {
                const output = document.getElementById("output");
                const els = document.embeds;
                const len = document.createElement("li");
                if (els && "length" in els && els.length) {
                    len.textContent = `There are ${els.length} embeds for document.embeds`;
                }

                output?.appendChild(len);
            }

            countDocumentEmbeds();
            const emb1 = document.getElementById("emb1");
            if (emb1) {
                emb1.remove();
            }
            countDocumentEmbeds();
        });
        const actual = await browser
            .$("#output")
            .getHTML({ includeSelectorTag: false, pierceShadowRoot: false });
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });
});
