import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer,
    "extension",
    new SourceAPI("extension")
);
__webpack_public_path__ = publicPath(window.RufflePlayer.config, "extension");

let uniqueMessageSuffix = null;
if (
    document.currentScript !== undefined &&
    document.currentScript !== null &&
    "src" in document.currentScript &&
    document.currentScript.src !== ""
) {
    // Default to the directory where this script resides.
    try {
        uniqueMessageSuffix = new URL(document.currentScript.src).searchParams.get(
            "uniqueMessageSuffix"
        );
    } catch (_) {}
}
if (uniqueMessageSuffix) {
    window.addEventListener("message", (event) => {
        // We only accept messages from ourselves.
        if (event.source !== window) {
            return;
        }

        const { type, index, data } = event.data;
        if (type === `FROM_RUFFLE${uniqueMessageSuffix}`) {
            // Ping back.
            window.postMessage({ type: `TO_RUFFLE${uniqueMessageSuffix}`, index, data }, "*");
        }
    });
}
