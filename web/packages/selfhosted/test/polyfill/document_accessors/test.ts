import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { use, expect } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Document accessor", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/document_accessors`);
    });

    it("Accesses the right content with ruffle", async () => {
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
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("ruffle-object#obj1"),
        );
        await browser.execute(() => {
            function countDocumentAccessorElements(name: string) {
                const output = document.getElementById("output");
                const els = (
                    document as unknown as Record<
                        string,
                        HTMLElement | HTMLCollectionOf<HTMLElement>
                    >
                )[name];
                const len = document.createElement("li");
                const listEach = document.createElement("li");
                let totalEmbeds = 0;
                let totalObjects = 0;

                if (!els) {
                    len.textContent = `document["${name}"] returns 0 elements`;
                }
                if (els instanceof HTMLElement && els.nodeName === "EMBED") {
                    len.textContent = `document["${name}"] returns 1 element`;
                    totalEmbeds++;
                }
                if (els instanceof HTMLElement && els.nodeName === "OBJECT") {
                    len.textContent = `document["${name}"] returns 1 element`;
                    totalObjects++;
                }
                if (els && "length" in els && els.length) {
                    len.textContent = `document["${name}"] returns ${els.length} elements`;
                    for (let i = 0; i < els.length; i++) {
                        if (els.item(i)!.nodeName === "EMBED") {
                            totalEmbeds++;
                        }
                        if (els.item(i)!.nodeName === "OBJECT") {
                            totalObjects++;
                        }
                    }
                }

                listEach.textContent = `There are ${totalEmbeds} embeds and ${totalObjects} objects for document["${name}"]`;

                output?.appendChild(len);
                output?.appendChild(listEach);
            }

            countDocumentAccessorElements("fl");

            const emb1 = document.getElementById("emb1") as HTMLEmbedElement;
            const emb2 = document.getElementById("emb2") as HTMLEmbedElement;
            if (emb1) {
                emb1.name = "test";
            }
            countDocumentAccessorElements("test");
            countDocumentAccessorElements("fl");

            if (emb2) {
                emb2.setAttribute("name", "test");
            }
            countDocumentAccessorElements("test");
            countDocumentAccessorElements("fl");

            if (emb2) {
                emb2.name = "fl";
            }
            countDocumentAccessorElements("test");
            countDocumentAccessorElements("fl");

            if (emb1) {
                emb1.remove();
            }
            countDocumentAccessorElements("test");
            countDocumentAccessorElements("fl");
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
