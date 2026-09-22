import { enableBrowserOnOutdatedChromium, setPageLanguage } from "./utils";
import { bindOptions, resetOptions } from "./common";
import { buildInfo } from "ruffle-core";

window.addEventListener("DOMContentLoaded", async () => {
    enableBrowserOnOutdatedChromium();
    const data = await browser.storage.sync.get({
        responseHeadersUnsupported: false,
    });
    if (data["responseHeadersUnsupported"]) {
        document
            .getElementById("swf_takeover")!
            .parentElement!.classList.add("hidden");
    }
    setPageLanguage();
    document.title = browser.i18n.getMessage("settings_page");
    {
        const vt = document.getElementById("version-text")!;
        vt.textContent = buildInfo.versionName;
    }
    {
        const ao = document.getElementById("advanced-options")!;
        ao.textContent = browser.i18n.getMessage("settings_advanced_options");
    }
    {
        const rs = document.getElementById("reset-settings")!;
        rs.textContent = browser.i18n.getMessage("settings_reset");
        rs.addEventListener("click", async () => {
            if (confirm(browser.i18n.getMessage("settings_reset_confirm"))) {
                await resetOptions();
                window.location.reload();
            }
        });
    }
    bindOptions();
});
