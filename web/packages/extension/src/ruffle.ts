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
if (document.currentScript !== undefined && document.currentScript !== null) {
    polyfillCurrentScript();
    if (determineKnownTypeHasNoEmptyStringProp(document.currentScript, "src")) {
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

function polyfillCurrentScript(): void {
    const script = document.currentScript!;
    try {
        if (determineKnownTypeHasNoEmptyStringProp(script, "src")) {
            determineScriptIsMaskedSrcAndMakePolyfill(script);
        }
    } catch (_) {
        // Continue to run
    }
}

function determineScriptIsMaskedSrcAndMakePolyfill(
    script: HTMLScriptElement,
): void {
    const scriptUrlPolyfill = script.getAttribute("ruffle-src-polyfill");
    if (!scriptUrlPolyfill) {
        return;
    }
    const scriptUrl = script.src;
    const applyResetWebpackPublicPath = (overridePublicPath: string): void => {
        __webpack_public_path__ = overridePublicPath;
        // TODO: If there are other scripts that need to be dynamically created and executed, the current polyfill logic should also be applied.
    };
    // Reset webpack public path should be safe when it is using mask url
    if ("webkit-masked-url://hidden/" === scriptUrl) {
        const webpackCurrentPublicPath = __webpack_public_path__;
        const scriptAutoPublicPath =
            getWebpackPublicPathFromScriptSrc(scriptUrl);
        if (webpackCurrentPublicPath === scriptAutoPublicPath) {
            applyResetWebpackPublicPath(
                getWebpackPublicPathFromScriptSrc(scriptUrlPolyfill),
            );
        }
    }
    // TODO: Process other situations when mask url not webkit-masked-url://hidden/
}

// Exclude types that do not have this prop
function determineKnownTypeHasNoEmptyStringProp<
    T extends object,
    TKey extends PropertyKey,
>(obj: T, prop: TKey): obj is T extends Record<TKey, string> ? T : never {
    return prop in obj && (obj as Record<PropertyKey, unknown>)[prop] !== "";
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
