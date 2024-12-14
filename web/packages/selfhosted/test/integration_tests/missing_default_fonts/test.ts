import { injectRuffleAndWait, openTest, playAndMonitor } from "../../utils.js";
import { use } from "chai";
import chaiHtml from "chai-html";

use(chaiHtml);

describe("Missing Default Fonts", () => {
    it("load the test", async () => {
        await openTest(browser, "integration_tests/missing_default_fonts");
        await injectRuffleAndWait(browser);
        const player = await browser.$("<ruffle-object>");
        await playAndMonitor(browser, player, "Loaded!\n");
    });
});
