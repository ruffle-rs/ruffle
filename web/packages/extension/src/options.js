import { getI18Message } from "./utils";
import { bindBooleanOptions } from "./common";

window.addEventListener("DOMContentLoaded", () => {
    document.title = getI18Message("settings_page");
    bindBooleanOptions(["ruffle_enable", "ignore_optout"]);
});
