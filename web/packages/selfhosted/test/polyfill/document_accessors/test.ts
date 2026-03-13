import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect } from "chai";

describe("Document accessor", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/document_accessors`);
    });

    it("Accesses the right content with ruffle", async () => {
        async function setNameAttr(selector: string, value: string) {
            const el = await $(selector);
            await browser.execute(
                (element, val) => {
                    element.setAttribute("name", val);
                },
                el,
                value,
            );
        }

        async function setName(selector: string, value: string) {
            const el = await $(selector);
            await browser.execute(
                (element, val) => {
                    (element as HTMLEmbedElement | HTMLObjectElement).name =
                        val;
                },
                el,
                value,
            );
        }

        async function removeEl(selector: string) {
            const el = await $(selector);
            await browser.execute((element) => {
                element.remove();
            }, el);
        }

        async function getAccessorInfo(name: string) {
            return await browser.execute((name) => {
                const value = (
                    document as unknown as Record<
                        string,
                        HTMLElement | HTMLCollectionOf<HTMLElement>
                    >
                )[name];
                const result = { embeds: 0, objects: 0, length: 0, type: "" };

                if (!value) {
                    return result;
                }

                if (value instanceof HTMLElement) {
                    result.length = 1;
                    result.type = value.nodeName;
                    if (value.nodeName === "EMBED") {
                        result.embeds = 1;
                    }
                    if (value.nodeName === "OBJECT") {
                        result.objects = 1;
                    }
                    return result;
                }

                if ("length" in value) {
                    result.type = "HTMLCollectionLike";
                    result.length = value.length;
                    for (let i = 0; i < value.length; i++) {
                        const node = value.item(i);
                        if (node?.nodeName === "EMBED") {
                            result.embeds++;
                        }
                        if (node?.nodeName === "OBJECT") {
                            result.objects++;
                        }
                    }
                }

                return result;
            }, name);
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
        await playAndMonitor(
            browser,
            await browser.$("#test-container").$("ruffle-object#obj1"),
        );

        //
        // Before changes
        //
        const fl = await getAccessorInfo("fl");
        expect(fl.type).to.equal("HTMLCollectionLike");
        expect(fl.length).to.equal(4);
        expect(fl.embeds).to.equal(3);
        expect(fl.objects).to.equal(1);

        //
        // Change emb1.name = "test"
        //
        await setName("#emb1", "test");
        const test1 = await getAccessorInfo("test");
        const fl1 = await getAccessorInfo("fl");

        expect(test1.type).to.equal("EMBED");
        expect(test1.length).to.equal(1);
        expect(test1.embeds).to.equal(1);
        expect(test1.objects).to.equal(0);

        expect(fl1.type).to.equal("HTMLCollectionLike");
        expect(fl1.length).to.equal(3);
        expect(fl1.embeds).to.equal(2);
        expect(fl1.objects).to.equal(1);

        //
        // Change emb2.setAttribute("name", "test")
        //
        await setNameAttr("#emb2", "test");
        const test2 = await getAccessorInfo("test");
        const fl2 = await getAccessorInfo("fl");

        expect(test2.type).to.equal("HTMLCollectionLike");
        expect(test2.length).to.equal(2);
        expect(test2.embeds).to.equal(2);
        expect(test2.objects).to.equal(0);

        expect(fl2.type).to.equal("HTMLCollectionLike");
        expect(fl2.length).to.equal(2);
        expect(fl2.embeds).to.equal(1);
        expect(fl2.objects).to.equal(1);

        //
        // Move emb2 back to name="fl"
        //
        await setName("#emb2", "fl");
        const test3 = await getAccessorInfo("test");
        const fl3 = await getAccessorInfo("fl");

        expect(test3.type).to.equal("EMBED");
        expect(test3.length).to.equal(1);
        expect(test3.embeds).to.equal(1);
        expect(test3.objects).to.equal(0);

        expect(fl3.type).to.equal("HTMLCollectionLike");
        expect(fl3.length).to.equal(3);
        expect(fl3.embeds).to.equal(2);
        expect(fl3.objects).to.equal(1);

        //
        // Remove emb1
        //
        await removeEl("#emb1");
        const test4 = await getAccessorInfo("test");
        const fl4 = await getAccessorInfo("fl");

        expect(test4.type).to.equal("");
        expect(test4.length).to.equal(0);
        expect(fl4.type).to.equal("HTMLCollectionLike");
        expect(fl4.length).to.equal(3);
        expect(fl4.embeds).to.equal(2);
        expect(fl4.objects).to.equal(1);
    });
});
