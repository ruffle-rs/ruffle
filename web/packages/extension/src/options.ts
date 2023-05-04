import * as utils from "./utils";
import { bindOptions } from "./common";

window.addEventListener("DOMContentLoaded", () => {
    document.title = utils.i18n.getMessage("settings_page");
    document.getElementById("advanced_options")!.textContent =
        utils.i18n.getMessage("settings_advanced_options");
    bindOptions();
});
