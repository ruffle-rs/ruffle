import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

describe("Context Menu", () => {
    it("(quirks mode) load the test", async () => {
        await browser.setWindowSize(1000, 1000);

        await openTest(
            browser,
            "integration_tests/context_menu_escaping_body",
            "index_quirks.html",
        );
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded!\n");

        // Dismiss hardware acceleration modal in Chrome
        await player.click();
        await player.click();
    });

    it("(quirks mode) open context menu", async () => {
        const player = await browser.$("#objectElement");

        await player.click({ x: -150, y: -150, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        expect(menuLocation.x).to.equal(50);
        expect(menuLocation.y).to.equal(50);

        // Dismiss the menu
        await player.click({ x: -151, y: -151 });
    });

    it("(no quirks mode) load the test", async () => {
        await openTest(
            browser,
            "integration_tests/context_menu_escaping_body",
            "index_no_quirks.html",
        );
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded!\n");

        // Dismiss hardware acceleration modal in Chrome
        await player.click();
        await player.click();
    });

    it("(no quirks mode) open context menu", async () => {
        const player = await browser.$("#objectElement");

        await player.click({ x: -150, y: -150, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        expect(menuLocation.x).to.equal(50);
        expect(menuLocation.y).to.equal(50);

        // Dismiss the menu
        await player.click({ x: -151, y: -151 });
    });
});
