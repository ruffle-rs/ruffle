import fs from "fs";
import { FluentBundle, FluentResource } from "@fluent/bundle";
import { expect } from "chai";
import { Setup } from "ruffle-core";
import { injectRuffleAndWait } from "../../utils.js";

const textsDir = new URL("../../../../core/texts/", import.meta.url);

/**
 * Reads the text with the given ID from the first locale in the chain that has it.
 *
 * @param localeChain Locales to look the text up in, in priority order
 * @param id ID of the text
 * @returns The text
 */
function expectedText(localeChain: string[], id: string): string {
    for (const locale of localeChain) {
        const bundle = new FluentBundle(locale);
        const dir = new URL(`${locale}/`, textsDir);
        for (const file of fs.readdirSync(dir)) {
            if (file.endsWith(".ftl")) {
                const source = fs.readFileSync(new URL(file, dir), "utf8");
                bundle.addResource(new FluentResource(source));
            }
        }
        const message = bundle.getMessage(id);
        if (message?.value) {
            return bundle.formatPattern(message.value);
        }
    }
    throw new Error(`No text for ${id} in ${localeChain}`);
}

/**
 * Overrides the browser's preferred languages.
 *
 * @param languages Value of `navigator.languages`
 */
async function setLanguages(languages: string[]) {
    await browser.execute((languages) => {
        Object.defineProperty(navigator, "languages", {
            get: () => languages,
            configurable: true,
        });
    }, languages);
}

/**
 * Creates a player in a browser with the given preferred languages.
 *
 * @param languages Value of `navigator.languages`
 * @param setLanguagesAfterImport Whether to set the languages after Ruffle is
 * imported, instead of before
 * @returns The player element
 */
async function createPlayer(
    languages: string[],
    setLanguagesAfterImport: boolean = false,
) {
    await browser.url("http://localhost:4567/test_assets/js_api.html");
    if (!setLanguagesAfterImport) {
        await setLanguages(languages);
    }
    await injectRuffleAndWait(browser);
    if (setLanguagesAfterImport) {
        await setLanguages(languages);
    }
    await browser.execute(() => {
        const ruffle = (window.RufflePlayer as Setup.PublicAPI).newest();
        const player = ruffle!.createPlayer();
        player.id = "ruffle-player";
        document.getElementById("test-container")!.appendChild(player);
    });
    return await browser.$("#ruffle-player").getElement();
}

async function shadowText(player: WebdriverIO.Element, selector: string) {
    return await (await player.shadow$(selector)).getProperty("textContent");
}

async function shadowTitle(player: WebdriverIO.Element, selector: string) {
    return await (await player.shadow$(selector)).getAttribute("title");
}

/**
 * Checks texts of elements localized in different ways: SVG text content,
 * HTML text content, and the title attribute.
 *
 * @param player The player element
 * @param localeChain Locales the texts are expected to come from, in priority order
 */
async function expectTexts(player: WebdriverIO.Element, localeChain: string[]) {
    expect(await shadowText(player, "#unmute-text")).to.equal(
        expectedText(localeChain, "click-to-unmute"),
    );
    expect(await shadowText(player, "#save-manager .modal-button")).to.equal(
        expectedText(localeChain, "save-backup-all"),
    );
    expect(await shadowTitle(player, "#volume-mute")).to.equal(
        expectedText(localeChain, "volume-controls-unmute"),
    );
    expect(await shadowTitle(player, "#volume-max")).to.equal(
        expectedText(localeChain, "volume-controls-mute"),
    );
}

describe("Localization", () => {
    it("uses en-US texts", async () => {
        const player = await createPlayer(["en-US"]);
        await expectTexts(player, ["en-US"]);
    });

    it("uses texts of the preferred locale", async () => {
        const player = await createPlayer(["pl-PL"]);
        await expectTexts(player, ["pl-PL", "en-US"]);
    });

    it("uses the locale preferred when the player is created", async () => {
        const player = await createPlayer(["pl-PL"], true);
        await expectTexts(player, ["pl-PL", "en-US"]);
    });

    it("matches a language without a region", async () => {
        const player = await createPlayer(["de"]);
        await expectTexts(player, ["de-DE", "en-US"]);
    });

    it("uses the first supported preferred locale", async () => {
        const player = await createPlayer(["xx-XX", "pl-PL", "de-DE"]);
        await expectTexts(player, ["pl-PL", "de-DE", "en-US"]);
    });

    it("falls back to en-US for unsupported locales", async () => {
        const player = await createPlayer(["xx-XX"]);
        await expectTexts(player, ["en-US"]);
    });

    it("falls back to en-US for texts missing in the locale", async () => {
        // Not every text is translated to every locale.
        const player = await createPlayer(["th-TH"]);
        await expectTexts(player, ["th-TH", "en-US"]);
    });
});
