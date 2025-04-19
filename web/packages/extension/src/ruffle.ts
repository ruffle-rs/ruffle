import { Setup } from "ruffle-core";
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
            Setup.installRuffle("extension");
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
if (document.currentScript instanceof HTMLScriptElement) {
    polyfillCurrentScriptWebpackPublicPath();
    if (document.currentScript.src) {
        try {
            ID = new URL(document.currentScript.src).searchParams.get("id");
        } catch (_) {
            // ID remains null.
        }
    }
    if (ID === null) {
        // Fallback to get id from attrs
        const ruffleId = document.currentScript.getAttribute("ruffle-id");
        if (ruffleId) {
            ID = ruffleId;
        }
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

/**
 * We are overriding the publicPath automatically configured by webpack here because the browser might mask the script's src attribute (for example, Safari 16+ may mask the script.src of an extension as webkit-masked-url://hidden/).
 */
function polyfillCurrentScriptWebpackPublicPath(): void {
    const script = document.currentScript as HTMLScriptElement;
    try {
        const scriptUrlPolyfill = script.getAttribute("ruffle-src-polyfill");
        if (!scriptUrlPolyfill) {
            return;
        }
        const scriptUrl = script.src;
        const scriptAutoPublicPath =
            getWebpackPublicPathFromScriptSrc(scriptUrl);

        if (
            scriptUrl === "webkit-masked-url://hidden/" &&
            __webpack_public_path__ === scriptAutoPublicPath
        ) {
            const polyfillPath =
                getWebpackPublicPathFromScriptSrc(scriptUrlPolyfill);
            // TODO: If there are other scripts that need to be dynamically created and executed, the current polyfill logic should also be applied.
            __webpack_public_path__ = polyfillPath;
        }
        // TODO: Process other situations when mask url not webkit-masked-url://hidden/
    } catch (_) {
        // Continue to run
    }
}

// Copied from Webpack: https://github.com/webpack/webpack/blob/f1bdec5cc70236083e45b665831d5d79d6485db7/lib/runtime/AutoPublicPathRuntimeModule.js#L75
function getWebpackPublicPathFromScriptSrc(scriptUrl: string): string {
    return scriptUrl
        .replace(/^blob:/, "")
        .replace(/#.*$/, "")
        .replace(/\?.*$/, "")
        .replace(/\/[^/]+$/, "/");
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
