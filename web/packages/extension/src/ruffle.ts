import { PublicAPI, SourceAPI, publicPath } from "ruffle-core";

const api = PublicAPI.negotiate(
    window.RufflePlayer!,
    "extension",
    new SourceAPI("extension")
);
window.RufflePlayer = api;
__webpack_public_path__ = publicPath(api.config);

let uniqueMessageSuffix: string | null = null;
if (
    document.currentScript !== undefined &&
    document.currentScript !== null &&
    "src" in document.currentScript &&
    document.currentScript.src !== ""
) {
    // Default to the directory where this script resides.
    try {
        uniqueMessageSuffix = new URL(
            document.currentScript.src
        ).searchParams.get("uniqueMessageSuffix");
    } catch (_) {
        // uniqueMessageSuffix remains null.
    }
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
            const message = {
                type: `TO_RUFFLE${uniqueMessageSuffix}`,
                index,
                data,
            };
            window.postMessage(message, "*");
        }
    });
}
