import { installRuffle } from "ruffle-core";
import { Message } from "./messages";

function handleMessage(message: Message) {
    switch (message.type) {
        case "load": {
            if (window.RufflePlayer === undefined) {
                window.RufflePlayer = {};
            }
            if (window.RufflePlayer.config === undefined) {
                window.RufflePlayer.config = {};
            }
            window.RufflePlayer.config = {
                ...message.config,
                ...window.RufflePlayer.config,
                openInNewTab,
            };
            installRuffle("extension");
            return {};
        }
        case "ping":
            // Ping back.
            return {};
        default:
            // Ignore unknown messages.
            return null;
    }
}

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

function openInNewTab(swf: URL): void {
    const message = {
        to: `ruffle_content${ID}`,
        index: null,
        data: {
            type: "open_url_in_player",
            url: swf.toString(),
        },
    };
    window.postMessage(message, "*");
}

if (ID) {
    window.addEventListener("message", (event) => {
        // We only accept messages from ourselves.
        if (event.source !== window || !event.data) {
            return;
        }

        const { to, index, data } = event.data;
        if (to === `ruffle_page${ID}`) {
            const response = handleMessage(data);
            if (response) {
                const message = {
                    to: `ruffle_content${ID}`,
                    index,
                    data: response,
                };
                window.postMessage(message, "*");
            }
        }
    });
}
