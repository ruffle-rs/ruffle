import { getI18nMessage } from "./utils";
import { bindBooleanOptions } from "./common";

window.addEventListener("DOMContentLoaded", () => {
    document.title = getI18nMessage("settings_page");
    bindBooleanOptions();
});
