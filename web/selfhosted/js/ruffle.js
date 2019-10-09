import { construct_public_api } from "../../js-src/public-api";
import { get_config_options } from "../../js-src/config";

let html = document.getElementsByTagName("html")[0];
let page_options = get_config_options(html);

window.RufflePlayer = window.RufflePlayer || {};
window.RufflePlayer.local = construct_public_api();

//TODO: proper version negotiation
if (window.RufflePlayer.init === undefined) {
    window.RufflePlayer.init = window.RufflePlayer.local.init;
}

//This is intended for sites that don't configure Ruffle themselves.
//If the page calls Ruffle before DOMContentLoaded, then we hold off on the
//standard set of interdictions.
window.addEventListener("DOMContentLoaded", function () {
    if (!window.RufflePlayer.invoked) {
        window.RufflePlayer.init(["plugin-detect", "static-content"]);
    }
})