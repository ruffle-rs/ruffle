import {
    getTraceOutput,
    hideHardwareAccelerationModal,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

describe("URL Rewrite Rules", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/url_rewrite_rules");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded test!\n");
        await hideHardwareAccelerationModal(browser, player);
    });

    it("rewrites URL of other1 to a relative one", async () => {
        const player = await browser.$("#objectElement");

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.focus();
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "A",
                    code: "A",
                    keyCode: 65,
                    bubbles: true,
                }),
            );
        }, player);

        expect(await getTraceOutput(browser, player)).to.equal(
            "Loaded other1!\nQP Value: example.com/other1\n",
        );
    });

    it("rewrites URL of other1 to an absolute one", async () => {
        const player = await browser.$("#objectElement");

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.focus();
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "B",
                    code: "B",
                    keyCode: 66,
                    bubbles: true,
                }),
            );
        }, player);

        expect(await getTraceOutput(browser, player)).to.equal(
            "Loaded other1!\nQP Value: http://localhost:4567/test/integration_tests/url_rewrite_rules/other1\n",
        );
    });

    it("does not rewrite URL of other2", async () => {
        const player = await browser.$("#objectElement");

        await browser.execute((element) => {
            const el = element as unknown as HTMLElement;
            el.focus();
            el.dispatchEvent(
                new KeyboardEvent("keydown", {
                    key: "C",
                    code: "C",
                    keyCode: 67,
                    bubbles: true,
                }),
            );
        }, player);

        expect(await getTraceOutput(browser, player)).to.equal(
            "Loaded other2!\n",
        );
    });

    it("rewrites URL of a clicked link", async () => {
        const player = await browser.$("#objectElement");

        player.click();

        await browser.waitUntil(
            async () => {
                return (await browser.getUrl()).startsWith(
                    "https://www.example.com",
                );
            },
            {
                timeoutMsg: "Expected window URL to change",
            },
        );

        expect(await browser.getUrl()).to.equal(
            "https://www.example.com/$1/$&",
        );
    });
});
