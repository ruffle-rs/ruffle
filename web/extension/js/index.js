import { interdict } from "../../js-src/interdiction";
import { get_config_options, DEFAULT_CONFIG } from "../../js-src/config";

let html = document.getElementsByTagName("html")[0];
let page_options = get_config_options(html);

if (!page_options.optout) {
    let interdictions = page_options.interdict || DEFAULT_CONFIG.interdict;
    interdict(interdictions);
} else {
    console.log("WebExtension Ruffle execution prohibited by page");
}