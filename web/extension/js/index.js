import { PublicAPI } from "../../js-src/public-api";
import { SourceAPI } from "../../js-src/source-api";
import { get_config_options } from "../../js-src/config";

let html = document.getElementsByTagName("html")[0];
let page_options = get_config_options(html);

if (!page_options.optout) {
    window.RufflePlayer = PublicAPI.negotiate(window.RufflePlayer, "extension", new SourceAPI());
} else {
    console.log("WebExtension Ruffle execution prohibited by page");
}