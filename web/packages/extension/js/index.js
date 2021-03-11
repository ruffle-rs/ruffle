import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI("extension")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "extension");

function getObfuscatedEventPrefix() {
    if (
        document.currentScript !== undefined &&
        document.currentScript !== null &&
        "src" in document.currentScript &&
        document.currentScript.src !== ""
    ) {
        // Default to the directory where this script resides.
        try {
            return new URL(document.currentScript.src).searchParams.get(
                "obfuscatedEventPrefix"
            );
        } catch (e) {
            return null;
        }
    }
}

const obfuscatedEventPrefix = getObfuscatedEventPrefix();
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
