import { PublicAPI } from "../../js-src/public-api";
import { SourceAPI } from "../../js-src/source-api";
import { get_config_options } from "../../js-src/config";

let html = document.getElementsByTagName("html")[0];
let page_options = get_config_options(html);

if (!page_options.optout) {
    window.RufflePlayer = PublicAPI.negotiate(window.RufflePlayer, "extension", new SourceAPI());

    //This is intended for sites that don't configure Ruffle themselves.
    //If the page calls Ruffle before DOMContentLoaded, then we hold off on the
    //standard set of interdictions.
    window.addEventListener("DOMContentLoaded", function () {
        window.RufflePlayer.init();
    });
} else {
    console.log("WebExtension Ruffle execution prohibited by page");
}