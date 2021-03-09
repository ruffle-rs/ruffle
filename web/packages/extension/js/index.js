import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI("extension")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "extension");

if (obfuscatedEventPrefix) {
    document.addEventListener(obfuscatedEventPrefix + "_request", function (e) {
        let body = JSON.parse(e.detail);
        let response = {};

        if (body.action === "get_page_options") {
            //response.pageOptions = pageOptions;
        }

        let event = new CustomEvent(obfuscatedEventPrefix + "_response", {
            detail: JSON.stringify(response),
        });
        document.dispatchEvent(event);
    });
}
