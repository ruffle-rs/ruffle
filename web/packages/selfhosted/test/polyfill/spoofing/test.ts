/* eslint-disable @typescript-eslint/no-unused-expressions */

import { openTest, injectRuffleAndWait } from "../../utils.js";
import { expect, use } from "chai";
import chaiHtml from "chai-html";
import fs from "fs";

use(chaiHtml);

describe("Spoofing is not easily detectable", () => {
    it("loads the test", async () => {
        await openTest(browser, `polyfill/spoofing`);
    });

    it("Polyfills", async () => {
        await injectRuffleAndWait(browser);
        await browser.$("<ruffle-object />").waitForExist();

        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(
            `${import.meta.dirname}/expected.html`,
            "utf8",
        );
        expect(actual).html.to.equal(expected);
    });

    it("Spoofs navigator.plugins", async () => {
        const names = await browser.execute(() => {
            const names = [];
            for (let i = 0; i < navigator.plugins.length; i++) {
                names.push(navigator.plugins[i]!.name);
            }
            return names;
        });
        expect(names).to.include("Shockwave Flash");

        const instance = await browser.execute(() => {
            return navigator.plugins instanceof PluginArray;
        });
        expect(instance).be.true;
    });

    it("Spoofs navigator.mimeTypes", async () => {
        const types = await browser.execute(() => {
            const types = [];
            for (let i = 0; i < navigator.mimeTypes.length; i++) {
                types.push(navigator.mimeTypes[i]!.type);
            }
            return types;
        });
        expect(types).to.include("application/x-shockwave-flash");

        const instance = await browser.execute(() => {
            for (let i = 0; i < navigator.mimeTypes.length; i++) {
                if (!(navigator.mimeTypes[i] instanceof MimeType)) {
                    return false;
                }
            }
            return navigator.mimeTypes instanceof MimeTypeArray;
        });
        expect(instance).be.true;
    });
});
