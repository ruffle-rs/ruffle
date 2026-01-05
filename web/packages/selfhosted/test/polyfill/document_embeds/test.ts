import { injectRuffleAndWait, openTest } from "../../utils.js";
import { expect } from "chai";

describe("Document embeds", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/document_embeds`);
    });

    it("accesses the right number of elements with ruffle", async () => {
        async function removeEl(selector: string) {
            const el = await $(selector);
            await browser.execute((element) => {
                element.remove();
            }, el);
        }

        await injectRuffleAndWait(browser);
        await browser
            .$("#test-container")
            .$("ruffle-embed#emb1")
            .waitForExist();
        await browser
            .$("#test-container")
            .$("ruffle-embed#emb2")
            .waitForExist();
        await browser
            .$("#test-container")
            .$("ruffle-embed#emb3")
            .waitForExist();

        expect(
            await browser.execute(() => document.embeds === document.embeds),
        ).to.equal(true);

        expect(
            await browser.execute(() => document.embeds?.length ?? 0),
        ).to.equal(3);

        await removeEl("#emb1");

        expect(
            await browser.execute(() => document.embeds?.length ?? 0),
        ).to.equal(2);

        await browser.execute(() => {
            const embed = document.createElement("embed");
            embed.src = "about:blank";
            embed.type = "text/html";
            document.body.appendChild(embed);
        });

        expect(
            await browser.execute(() => document.embeds?.length ?? 0),
        ).to.equal(3);
    });
});
