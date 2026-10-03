import { enableBrowserOnOutdatedChromium, setPageLanguage } from "./utils";
import { buildInfo } from "ruffle-core";

window.addEventListener("DOMContentLoaded", () => {
    enableBrowserOnOutdatedChromium();
    setPageLanguage();
    document.title = browser.i18n.getMessage("onboarding_page");
    {
        const vt = document.getElementById("version-text")!;
        vt.textContent = buildInfo.versionName;
    }
    {
        const pe = document.getElementById("permissions-explanation")!;
        pe.textContent = browser.i18n.getMessage("permissions_explanation");
        const gp = document.getElementById("grant-permissions")!;
        gp.textContent = browser.i18n.getMessage("permissions_grant");
        gp.addEventListener("click", async () => {
            const granted = await browser.permissions.request({
                origins: ["<all_urls>"],
            });
            if (granted) {
                window.close();
            } else {
                alert(browser.i18n.getMessage("permissions_not_granted"));
            }
        });
    }
});
