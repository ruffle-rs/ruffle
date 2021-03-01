/**
 * Pierce the extension sandbox by copying our code into main world.
 *
 * The isolation extension content scripts get is neat, but it causes problems
 * based on what browser you use:
 *
 * 1. On Chrome, you are explicitly banned from registering custom elements.
 * 2. On Firefox, you can register custom elements but they can't expose any
 *    useful API surface, and can't even see their own methods.
 *
 * This code exists to pierce the extension sandbox, while maintaining:
 *
 * 1. The isolation of not interfering with the page's execution environment
 *    unintentionally.
 * 2. The ability to load extension resources such as .wasm files.
 *
 * We also provide a content script message listener that proxies messages
 * into/from the main world.
 */

import * as utils from "./utils";

const pendingMessages = [];

/**
 * Send a message to the main world, where Ruffle runs.
 * @param {*} data - JSON-serializable data to send to main world.
 * @returns {Promise<*>} JSON-serializable response from main world.
 */
function sendMessageToPage(data) {
    const message = {
        type: "FROM_RUFFLE",
        index: pendingMessages.length,
        data,
    };
    window.postMessage(message, "*");
    return new Promise((resolve, reject) => {
        pendingMessages.push({ resolve, reject });
    });
}

/**
 * Inject a raw script to the main world.
 * @param {string} src - Script to inject.
 */
function injectScriptRaw(src) {
    const script = document.createElement("script");
    script.textContent = src;
    (document.head || document.documentElement).append(script);
}

/**
 * Inject a script by URL to the main world.
 * @param {string} url - Script URL to inject.
 */
function injectScriptURL(url) {
    const script = document.createElement("script");
    script.src = url;
    (document.head || document.documentElement).append(script);
}

/**
 * Check whether the current page (or one of its ancestors) is configured
 * to opt-out from Ruffle.
 * @returns {boolean} Whether the current page opts-out or not.
 */
function checkPageOptout() {
    if (document.documentElement.hasAttribute("data-ruffle-optout")) {
        return true;
    }
    try {
        if (
            window.top &&
            window.top.document &&
            window.top.document.documentElement &&
            window.top.document.documentElement.hasAttribute(
                "data-ruffle-optout"
            )
        ) {
            // In case the opting-out page uses iframes.
            return true;
        }
    } catch (e) {
        console.warn(`Unable to check top-level optout: ${e.message}`);
    }
    return false;
}

(async () => {
    const options = await utils.getOptions(["ruffleEnable", "ignoreOptout"]);
    const pageOptout = checkPageOptout();
    const shouldLoad =
        options.ruffleEnable &&
        !window.RufflePlayer &&
        (options.ignoreOptout || !pageOptout);

    utils.runtime.onMessage.addListener((message, sender, sendResponse) => {
        if (shouldLoad) {
            sendMessageToPage(message).then((response) => {
                sendResponse({
                    loaded: true,
                    tabOptions: options,
                    optout: pageOptout,
                    data: response,
                });
            });
            return true;
        } else {
            sendResponse({
                loaded: false,
                tabOptions: options,
                optout: pageOptout,
            });
            return false;
        }
    });

    if (shouldLoad) {
        // We must run the plugin polyfill before any flash detection scripts.
        // Unfortunately, this might still be too late for some websites when using Chrome (#969).
        injectScriptRaw(require("./pluginPolyfill")); // TODO: use plugin-polyfill.ts
        injectScriptURL(utils.runtime.getURL("dist/ruffle.js"));

        window.addEventListener("message", (event) => {
            // We only accept messages from ourselves.
            if (event.source !== window) {
                return;
            }

            const { type, index, data } = event.data;
            if (type === "TO_RUFFLE") {
                const request = pendingMessages[index];
                if (request) {
                    pendingMessages[index] = null;
                    request.resolve(data);
                } else {
                    // TODO: Handle page-initiated messages.
                    console.warn("no pending requests.");
                }
            }
        });
    }
})();
