import { PublicAPI, SourceAPI, Config } from "ruffle-core";

interface LoadMessage {
    type: "load";
    config: Config;
}

interface PingMessage {
    type: "ping";
}

type Message = LoadMessage | PingMessage;

function handleMessage(message: Message) {
    switch (message.type) {
        case "load":
            window.RufflePlayer = window.RufflePlayer || {};
            window.RufflePlayer.config = {
                ...window.RufflePlayer.config,
                ...message.config,
            };
            window.RufflePlayer = PublicAPI.negotiate(
                window.RufflePlayer!,
                "extension",
                new SourceAPI("extension")
            );
            return {};
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

if (ID) {
    window.addEventListener("message", (event) => {
        // We only accept messages from ourselves.
        if (event.source !== window) {
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
