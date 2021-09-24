import { PublicAPI, SourceAPI } from "ruffle-core";

window.RufflePlayer = PublicAPI.negotiate(
    window.RufflePlayer!,
    "extension",
    new SourceAPI("extension")
);

let ID: string | null = null;
if (
    document.currentScript !== undefined &&
    document.currentScript !== null &&
    "src" in document.currentScript &&
    document.currentScript.src !== ""
) {
    try {
        ID = new URL(document.currentScript.src).searchParams.get("id");
    } catch (_) {
        // ID remains null.
    }
}

if (ID) {
    window.addEventListener("message", (event) => {
        // We only accept messages from ourselves.
        if (event.source !== window) {
            return;
        }

        const { to, index, data } = event.data;
        if (to === `ruffle_page${ID}`) {
            // Ping back.
            const message = {
                to: `ruffle_content${ID}`,
                index,
                data,
            };
            window.postMessage(message, "*");
        }
    });
}
