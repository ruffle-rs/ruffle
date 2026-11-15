import {
    injectRuffleAndWait,
    openTest,
    waitForPlayerToLoad,
} from "../../utils.js";
import { expect } from "chai";
import { Player } from "ruffle-core";

describe("Object and embed options", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/object_embed_options`);
        await injectRuffleAndWait(browser);
    });

    const options = {
        allowFullscreen: false,
        allowScriptAccess: false,
        menu: false,
        quality: "low",
        scale: "noScale",
        wmode: "transparent",
    } as const;

    for (const [id, description] of [
        ["object-attributes", "loads the options from object attributes"],
        ["embed-attributes", "loads the options from embed attributes"],
        ["object-params", "loads the options from object params"],
        ["object-params-override", "object params override object attributes"],
    ] as const) {
        it(description, async () => {
            const player = await browser.$(`#${id}`).getElement();
            await player.waitForExist();
            await waitForPlayerToLoad(browser, player);
            const actual = await browser.execute((playerElement) => {
                const player = playerElement as Player.PlayerElement;
                return player.ruffle().loadedConfig;
            }, player);
            expect(actual).to.include(options);
        });
    }
});
