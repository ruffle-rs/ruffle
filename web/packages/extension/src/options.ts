import * as utils from "./utils";
import { bindOptions, resetOptions } from "./common";
import { buildInfo } from "ruffle-core";

window.addEventListener("DOMContentLoaded", async () => {
    const data = await utils.storage.sync.get({
        responseHeadersUnsupported: false,
    });
    if (data["responseHeadersUnsupported"]) {
        document
            .getElementById("swf_takeover")!
            .parentElement!.classList.add("hidden");
    }
    document.title = utils.i18n.getMessage("settings_page");
    {
        const vt = document.getElementById("version-text")!;
        vt.textContent = buildInfo.versionName;
    }
    {
        const ao = document.getElementById("advanced-options")!;
        ao.textContent = utils.i18n.getMessage("settings_advanced_options");
    }
    {
        const rs = document.getElementById("reset-settings")!;
        rs.textContent = utils.i18n.getMessage("settings_reset");
        rs.addEventListener("click", async () => {
            if (confirm(utils.i18n.getMessage("settings_reset_confirm"))) {
                await resetOptions();
                window.location.reload();
            }
        });
    }
    bindOptions();
});
