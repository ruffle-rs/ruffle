import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

async function getViewportWidth(): Promise<number> {
    return await browser.execute(() => {
        return window.innerWidth;
    });
}

describe("Context Menu", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/context_menu_position");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded!\n");

        // Dismiss hardware acceleration modal in Chrome
        await player.click();
        await player.click();

        // Make the window large enough to accommodate a protruding context menu
        await browser.setWindowSize(1000, 1000);
    });

    it("open context menu in the middle LTR", async () => {
        const player = await browser.$("#objectElement");

        await player.click({ x: 0, y: 0, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        expect(menuLocation.x).to.equal(500);
        expect(menuLocation.y).to.equal(500);

        // Dismiss the menu
        await player.click({ x: -10, y: -10 });
    });

    it("open context menu in the corner LTR", async () => {
        const player = await browser.$("#objectElement");

        await player.click({ x: 150, y: 150, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        expect(menuLocation.x).to.equal(650);
        expect(menuLocation.y).to.equal(650);

        // Dismiss the menu
        await player.click({ x: -10, y: -10 });
    });

    it("switch LTR -> RTL", async () => {
        await browser.execute(() => {
            document.documentElement.setAttribute("dir", "rtl");
        });
    });

    it("open context menu in the middle RTL", async () => {
        const player = await browser.$("#objectElement");
        const viewportWidth = await getViewportWidth();

        await player.click({ x: 0, y: 0, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();

        expect(menuLocation.x).to.equal(viewportWidth - 500);
        expect(menuLocation.y).to.equal(500);

        // Dismiss the menu
        await player.click({ x: -10, y: -10 });
    });

    it("open context menu in the corner RTL", async () => {
        const player = await browser.$("#objectElement");
        const viewportWidth = await getViewportWidth();

        await player.click({ x: -150, y: 150, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        expect(menuLocation.x).to.equal(viewportWidth - 650);
        expect(menuLocation.y).to.equal(650);

        // Dismiss the menu
        await player.click({ x: -10, y: -10 });
    });
});
