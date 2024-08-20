/* eslint-disable @typescript-eslint/no-unused-expressions */

import {
    getTraceOutput,
    injectRuffleAndWait,
    openTest,
    playAndMonitor,
} from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import { Key } from "webdriverio";

use(chaiHtml);

async function supportsClipboardReadText(): Promise<boolean> {
    return await browser.execute(async () => {
        return (
            navigator.clipboard &&
            typeof navigator.clipboard.readText === "function"
        );
    });
}

async function focusFlashInput(player: ChainablePromiseElement) {
    await player.click({ x: 10 - 200, y: 110 - 200 });
    expect(await getTraceOutput(browser, player)).to.equal(
        "onMouseDown()\nonMouseUp()\n",
    );
}

async function focusHtmlInput() {
    await browser.execute(() => {
        const input = document.getElementById("inputElement")!;
        input.focus();
    });
}

async function openContextMenu(player: ChainablePromiseElement) {
    await player.click({ x: 10 - 200, y: 10 - 200, button: "right" });
}

async function openContextMenuOnInput(player: ChainablePromiseElement) {
    await player.click({ x: 10 - 200, y: 110 - 200, button: "right" });
}

async function clickContextMenuEntry(
    player: ChainablePromiseElement,
    text: string,
    button: string = "left",
) {
    const contextMenu = await player.shadow$("#context-menu");
    const item = await contextMenu.$(`.menu-item[data-text="${text}"]`);
    await item.click({ button });
}

describe("Context Menu", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/context_menu");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded!\n");

        // Dismiss hardware acceleration modal in Chrome
        await player.click();
        await player.click();
        await getTraceOutput(browser, player);

        // Chrome requires this
        browser.setPermissions({ name: "clipboard-read" }, "granted");
    });

    it("clicking out of context menu does not fire events", async () => {
        const player = await browser.$("#objectElement");

        await player.click({ x: 10 - 200, y: 10 - 200 });

        expect(await getTraceOutput(browser, player)).to.equal(
            "onMouseDown()\nonMouseUp()\n",
        );

        await player.click({ x: 20 - 200, y: 20 - 200, button: "right" });
        await player.click({ x: 10 - 200, y: 10 - 200 });
        await player.click({ x: 10 - 200, y: 10 - 200 });

        // Only one click should be logged.
        expect(await getTraceOutput(browser, player)).to.equal(
            "onMouseDown()\nonMouseUp()\n",
        );
    });

    it("left clicking a context menu entry works", async () => {
        const player = await browser.$("#objectElement");

        await browser.keys("q");

        expect(await getTraceOutput(browser, player)).to.equal(
            "quality: HIGH\n",
        );

        await openContextMenu(player);
        await clickContextMenuEntry(player, "Quality: Low", "left");

        await browser.keys("q");

        expect(await getTraceOutput(browser, player)).to.equal(
            "quality: LOW\n",
        );
    });

    it("right clicking a context menu entry works", async () => {
        const player = await browser.$("#objectElement");

        await browser.keys("q");

        expect(await getTraceOutput(browser, player)).to.equal(
            "quality: HIGH\n",
        );

        await openContextMenu(player);
        await clickContextMenuEntry(player, "Quality: Low", "right");

        await browser.keys("q");

        expect(await getTraceOutput(browser, player)).to.equal(
            "quality: LOW\n",
        );
    });

    it("copying text works", async () => {
        const player = await browser.$("#objectElement");

        // Populate text input in Flash
        await browser.keys("t");
        expect(await getTraceOutput(browser, player)).to.equal(
            "populating text\ntext changed: texample\n",
        );

        await focusFlashInput(player);

        // Select all
        await openContextMenuOnInput(player);
        await clickContextMenuEntry(player, "Select All");

        // Copy
        await openContextMenuOnInput(player);
        await clickContextMenuEntry(player, "Copy");

        // Paste
        await focusHtmlInput();
        await browser.keys([Key.Ctrl, "v"]);

        // Check what's inside
        const pastedText = await browser.execute(() => {
            const input = document.getElementById(
                "inputElement",
            )! as HTMLInputElement;
            const pastedText = input.value;
            input.value = "";
            return pastedText;
        });

        expect(pastedText).to.equal("texample");
    });

    it("cutting text works", async () => {
        const player = await browser.$("#objectElement");

        await focusFlashInput(player);

        // Select all
        await openContextMenuOnInput(player);
        await clickContextMenuEntry(player, "Select All");

        // Cut
        await openContextMenuOnInput(player);
        await clickContextMenuEntry(player, "Cut");

        expect(await getTraceOutput(browser, player)).to.equal(
            "text changed: \n",
        );

        // Paste
        await focusHtmlInput();
        await browser.keys([Key.Ctrl, "v"]);

        // Check what's inside
        const pastedText = await browser.execute(() => {
            const input = document.getElementById(
                "inputElement",
            )! as HTMLInputElement;
            return input.value;
        });

        expect(pastedText).to.equal("texample");
    });

    it("pasting text works", async function () {
        if (!(await supportsClipboardReadText())) {
            this.skip();
        }

        const player = await browser.$("#objectElement");

        // Populate text input in HTML
        await browser.execute(() => {
            const input = document.getElementById(
                "inputElement",
            )! as HTMLInputElement;
            input.value = "text to be pasted";
        });

        // Copy the text
        await focusHtmlInput();
        await browser.keys([Key.Ctrl, "a"]);
        await browser.keys([Key.Ctrl, "c"]);

        // Paste
        await focusFlashInput(player);
        await openContextMenuOnInput(player);
        await clickContextMenuEntry(player, "Paste");

        expect(await getTraceOutput(browser, player)).to.equal(
            "text changed: text to be pasted\n",
        );
    });

    it("a modal is shown that pasting is not supported", async function () {
        if (await supportsClipboardReadText()) {
            this.skip();
        }

        const player = await browser.$("#objectElement");

        // Try pasting
        await focusFlashInput(player);
        await openContextMenuOnInput(player);
        await clickContextMenuEntry(player, "Paste");

        const clipboardModal = await player.shadow$("#clipboard-modal");
        const clipboardModalVisible = await clipboardModal.isDisplayed();
        expect(clipboardModalVisible).to.be.true;

        const clipboardModalClose = await player.shadow$(
            "#clipboard-modal .close-modal",
        );
        await clipboardModalClose.click();

        const clipboardModal2 = await player.shadow$("#clipboard-modal");
        const clipboardModal2Visible = await clipboardModal2.isDisplayed();
        expect(clipboardModal2Visible).to.be.false;
    });
});
