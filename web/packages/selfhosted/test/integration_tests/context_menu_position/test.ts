import {
    assertNoMoreTraceOutput,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

async function getViewportWidth(): Promise<number> {
    return await browser.execute(() => {
        return window.innerWidth;
    });
}

async function setDirection(
    element: ChainablePromiseElement,
    direction: string,
) {
    await browser.execute(
        ({ el, dir }: { el: HTMLElement; dir: string }) => {
            el.dir = dir;
        },
        { el: element, dir: direction },
    );
}

describe("Context Menu", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/context_menu_position");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, ["Loaded!"]);

        // Dismiss hardware acceleration modal in Chrome
        await player.click();
        await player.click();

        // Make the window large enough to accommodate a protruding context menu
        await browser.setWindowSize(1500, 1500);
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

    it("switch context menu LTR -> RTL", async () => {
        // Note: normally we should change the preferred user language
        //   (navigator.language), but that's not easy without creating
        //   a new browser instance.

        const player = await browser.$("#objectElement");
        const menu = await player.$("#context-menu");
        await setDirection(menu, "rtl");
    });

    it("open RTL context menu in the middle RTL", async () => {
        const player = await browser.$("#objectElement");
        const viewportWidth = await getViewportWidth();

        await player.click({ x: 0, y: 0, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        const menuSize = await menu.getSize();
        expect(menuLocation.x).to.approximately(
            viewportWidth - 500 - menuSize.width,
            0.6,
        );
        expect(menuLocation.y).to.equal(500);

        // Dismiss the menu
        await player.click({ x: -10, y: -10 });
    });

    it("open RTL context menu in the corner RTL", async () => {
        const player = await browser.$("#objectElement");
        const viewportWidth = await getViewportWidth();

        await player.click({ x: -150, y: 150, button: "right" });

        const menu = await player.$("#context-menu");
        const menuLocation = await menu.getLocation();
        const menuSize = await menu.getSize();
        expect(menuLocation.x).to.approximately(
            viewportWidth - 650 - menuSize.width,
            0.6,
        );
        expect(menuLocation.y).to.equal(650);

        // Dismiss the menu
        await player.click({ x: -10, y: -10 });
    });

    it("no more traces", async function () {
        const player = await browser.$("#objectElement");
        assertNoMoreTraceOutput(browser, player);
    });
});
