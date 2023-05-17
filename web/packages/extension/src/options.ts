import * as utils from "./utils";
import { bindOptions, resetOptions } from "./common";

window.addEventListener("DOMContentLoaded", () => {
    document.title = utils.i18n.getMessage("settings_page");
    {
        const ao = document.getElementById("advanced_options")!;
        ao.textContent = utils.i18n.getMessage("settings_advanced_options");
    }
    {
        const rs = document.getElementById("reset_settings")!;
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
