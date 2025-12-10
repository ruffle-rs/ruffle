import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect } from "chai";

describe("Document embeds", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/document_embeds`);
    });

    it("Accesses the right number of elements with ruffle", async () => {
        async function countDocumentEmbeds() {
            return await browser.execute(() => {
                return document.embeds?.length ?? 0;
            });
        }

        async function removeEl(selector: string) {
            const el = await $(selector);
            await browser.execute((element) => {
                element.remove();
            }, el);
        }

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
        const embeds1 = await countDocumentEmbeds();
        expect(embeds1).to.equal(3);
        await removeEl("#emb1");
        const embeds2 = await countDocumentEmbeds();
        expect(embeds2).to.equal(2);
    });
});
