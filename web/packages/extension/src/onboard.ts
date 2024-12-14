import * as utils from "./utils";
import { buildInfo } from "ruffle-core";

window.addEventListener("DOMContentLoaded", () => {
    document.title = utils.i18n.getMessage("onboarding_page");
    {
        const vt = document.getElementById("version-text")!;
        vt.textContent = buildInfo.versionName;
    }
    {
        const pe = document.getElementById("permissions-explanation")!;
        pe.textContent = utils.i18n.getMessage("permissions_explanation");
        const gp = document.getElementById("grant-permissions")!;
        gp.textContent = utils.i18n.getMessage("permissions_grant");
        gp.addEventListener("click", async () => {
            const granted = await utils.permissions.request({
                origins: ["<all_urls>"],
            });
            if (granted) {
                window.close();
            } else {
                alert(utils.i18n.getMessage("permissions_not_granted"));
            }
        });
    }
});
