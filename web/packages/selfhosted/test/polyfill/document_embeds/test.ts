import { injectRuffleAndWait, openTest } from "../../utils.js";
import { expect } from "chai";

describe("Document embeds", () => {
    beforeEach(async () => {
        await openTest(browser, `polyfill/document_embeds`);
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
    });

    it("accesses the right number of elements with ruffle", async () => {
        async function removeEl(selector: string) {
            const el = await $(selector);
            await browser.execute((element) => {
                element.remove();
            }, el);
        }

        expect(
            await browser.execute(() => document.embeds === document.embeds),
        ).to.equal(true);

        expect(
            await browser.execute(() => document.embeds?.length ?? 0),
        ).to.equal(5);

        await removeEl("#emb1");

        expect(
            await browser.execute(() => document.embeds?.length ?? 0),
        ).to.equal(4);

        await browser.execute(() => {
            const embed = document.createElement("embed");
            embed.src = "about:blank";
            embed.type = "text/html";
            document.body.appendChild(embed);
        });

        expect(
            await browser.execute(() => document.embeds?.length ?? 0),
        ).to.equal(5);
    });

    it("supports index-based access", async () => {
        const ids = await browser.execute(() => [
            document.embeds.item(0)?.id,
            document.embeds[1]?.id,
            document.embeds[2]?.id,
            document.embeds.item(3)?.id,
            document.embeds[4]?.id,
        ]);

        expect(ids).to.deep.equal([
            "emb1",
            "emb2",
            "basic_emb1",
            "emb3",
            "basic_emb2",
        ]);
    });

    it("supports namedItem(name)", async () => {
        const result = await browser.execute(() => {
            return {
                alpha: document.embeds.namedItem("alpha")?.id,
                beta: document.embeds.namedItem("beta")?.id,
                delta: document.embeds.namedItem("delta")?.id,
                missing: document.embeds.namedItem("nope"),
            };
        });

        expect(result).to.deep.equal({
            alpha: "emb1",
            beta: "emb2", // first match
            delta: "basic_emb2",
            missing: null,
        });
    });

    it("namedItem falls back to id", async () => {
        const idMatch = await browser.execute(
            () => document.embeds.namedItem("emb3")?.id,
        );

        expect(idMatch).to.equal("emb3");
        const idMatch2 = await browser.execute(() =>
            document.embeds.namedItem("basic_emb2")?.getAttribute("name"),
        );

        expect(idMatch2).to.equal("delta");
    });

    it("is iterable", async () => {
        const ids = await browser.execute(() => {
            const result = [];
            for (const el of document.embeds) {
                result.push(el.id);
            }
            return result;
        });

        expect(ids).to.deep.equal([
            "emb1",
            "emb2",
            "basic_emb1",
            "emb3",
            "basic_emb2",
        ]);
    });

    it("is iterable using in", async () => {
        const ids = await browser.execute(() => {
            const result = [];
            for (const id in document.embeds) {
                result.push(id);
            }
            return result;
        });

        ids.sort();

        expect(ids).to.deep.equal([
            "0",
            "1",
            "2",
            "3",
            "4",
            "item",
            "length",
            "namedItem",
        ]);
    });

    it("Can use Object.keys", async () => {
        const ids = await browser.execute(() => {
            const result = [];
            for (const id of Object.keys(document.embeds)) {
                result.push(id);
            }
            return result;
        });

        expect(ids).to.deep.equal(["0", "1", "2", "3", "4"]);
    });

    it("Can use Object.values", async () => {
        const elIds = await browser.execute(() => {
            const result = [];
            for (const el of Object.values(document.embeds)) {
                result.push(el.id);
            }
            return result;
        });

        expect(elIds).to.deep.equal([
            "emb1",
            "emb2",
            "basic_emb1",
            "emb3",
            "basic_emb2",
        ]);
    });

    it("Can use Object.entries", async () => {
        const entries = await browser.execute(() => {
            const result = [];
            for (const [index, el] of Object.entries(document.embeds)) {
                result.push([index, el.id]);
            }
            return result;
        });

        expect(entries).to.deep.equal([
            ["0", "emb1"],
            ["1", "emb2"],
            ["2", "basic_emb1"],
            ["3", "emb3"],
            ["4", "basic_emb2"],
        ]);
    });

    it("updates index order after removal", async () => {
        await browser.execute(() => {
            document.getElementById("emb2")?.remove();
        });

        const ids = await browser.execute(() =>
            Array.from(document.embeds).map((e) => e.id),
        );

        expect(ids).to.deep.equal(["emb1", "basic_emb1", "emb3", "basic_emb2"]);
    });

    it("includes newly added embeds in order", async () => {
        await browser.execute(() => {
            const e = document.createElement("embed");
            e.id = "emb4";
            e.name = "gamma";
            e.src = "about:blank";
            document.body.appendChild(e);
        });

        const data = await browser.execute(() => ({
            length: document.embeds.length,
            lastId: document.embeds.item(5)?.id,
            gamma: document.embeds.namedItem("gamma")?.id,
        }));

        expect(data).to.deep.equal({
            length: 6,
            lastId: "emb4",
            gamma: "emb4",
        });
    });

    it("supports direct named access via proxy get", async () => {
        const result = await browser.execute(() => {
            // Explicitly declare the properties we intend to access to satisfy TS.
            type EmbedPolyfillTest = HTMLCollectionOf<HTMLEmbedElement> & {
                alpha?: Element;
                beta?: Element;
                delta?: Element;
                emb3?: Element;
                nope?: Element;
            };
            const embeds = document.embeds as unknown as EmbedPolyfillTest;

            return {
                alpha: embeds.alpha?.id,
                beta: embeds["beta"]?.id,
                delta: embeds.delta?.id,
                emb3: embeds.emb3?.id,
                missing: embeds.nope,
            };
        });

        expect(result).to.deep.equal({
            alpha: "emb1",
            beta: "emb2",
            delta: "basic_emb2",
            emb3: "emb3",
            missing: undefined,
        });
    });

    it("supports the 'in' operator for named embeds", async () => {
        const result = await browser.execute(() => {
            return {
                hasAlpha: "alpha" in document.embeds,
                hasEmb3: "emb3" in document.embeds,
                hasNope: "nope" in document.embeds,
                hasLength: "length" in document.embeds,
            };
        });

        expect(result).to.deep.equal({
            hasAlpha: true,
            hasEmb3: true,
            hasNope: false,
            hasLength: true,
        });
    });

    it("prioritizes native 'length' property over an element named 'length'", async () => {
        const result = await browser.execute(() => {
            const e = document.createElement("embed");
            e.id = "length_embed";
            e.name = "length";
            e.src = "about:blank";
            document.body.appendChild(e);

            const lengthValue = document.embeds["length"];
            const typeofLength = typeof lengthValue;

            const namedItemValue = document.embeds.namedItem("length")?.id;

            return {
                lengthValue,
                typeofLength,
                namedItemValue,
            };
        });

        expect(result).to.deep.equal({
            lengthValue: 6,
            typeofLength: "number",
            namedItemValue: "length_embed",
        });
    });
});
