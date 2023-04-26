import * as utils from "./utils";
import { bindOptions } from "./common";

window.addEventListener("DOMContentLoaded", () => {
    document.title = utils.i18n.getMessage("settings_page");
    const preferredRenderer = document.getElementById(
        "preferred_renderer"
    )! as HTMLSelectElement;
    bindOptions((options) => {
        preferredRenderer.disabled = !!options.forceRenderer;
    });
});
