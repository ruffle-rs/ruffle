/**
 *
 * This code provides a content script message listener that proxies messages
 * into/from the main world.
 *
 * On older Firefox, it also pierces the extension sandbox by copying our code into the main world
 *
 */

import * as utils from "./utils";
import { isMessage } from "./messages";

declare global {
    interface Navigator {
        // Only supported in Firefox, see https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Sharing_objects_with_page_scripts#accessing_page_script_objects_from_content_scripts
        wrappedJSObject?: Navigator;
    }
}

const pendingMessages: ({
    resolve(value: unknown): void;
    reject(reason?: unknown): void;
} | null)[] = [];

const ID = Math.floor(Math.random() * 100000000000);

/**
 * Send a message to the main world, where Ruffle runs.
 * @param {*} data - JSON-serializable data to send to main world.
 * @returns {Promise<*>} JSON-serializable response from main world.
 */
function sendMessageToPage(data: unknown): Promise<unknown> {
    const message = {
        to: "ruffle_page",
        index: pendingMessages.length,
        id: ID,
        data,
    };
    window.postMessage(message, "*");
    return new Promise((resolve, reject) => {
        pendingMessages.push({ resolve, reject });
    });
}

/**
 * Check whether the current page (or one of its ancestors) is configured
 * to opt-out from Ruffle.
 * @returns {boolean} Whether the current page opts-out or not.
 */
function checkPageOptout(): boolean {
    if (document.documentElement.hasAttribute("data-ruffle-optout")) {
        return true;
    }
    try {
        if (
            window.top &&
            window.top.document &&
            window.top.document.documentElement &&
            window.top.document.documentElement.hasAttribute(
                "data-ruffle-optout",
            )
        ) {
            // In case the opting-out page uses iframes.
            return true;
        }
    } catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        console.warn(`Unable to check top-level optout: ${message}`);
    }
    return false;
}

/**
 * @returns {boolean} Whether the current page is an XML document or not.
 */
function isXMLDocument(): boolean {
    // Based on https://developer.mozilla.org/en-US/docs/Web/API/Document/xmlVersion
    return document.createElement("foo").tagName !== "FOO";
}

(async () => {
    await utils.storage.sync.set({
        ["showReloadButton"]: false,
    });
    const options = await utils.getOptions();
    const explicitOptions = await utils.getExplicitOptions();

    const pageOptout = checkPageOptout();
    const shouldLoad =
        !isXMLDocument() &&
        options.ruffleEnable &&
        (options.ignoreOptout || !pageOptout);

    utils.runtime.onMessage.addListener((message, _sender, sendResponse) => {
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

    if (!shouldLoad) {
        return;
    }

    window.addEventListener("message", (event) => {
        // We only accept messages from ourselves.
        if (event.source !== window || !event.data) {
            return;
        }

        const { to, index, data, id } = event.data;
        if (to === "ruffle_content" && id === ID) {
            const request = index !== null ? pendingMessages[index] : null;
            if (request) {
                pendingMessages[index] = null;
                request.resolve(data);
            } else if (isMessage(data)) {
                switch (data.type) {
                    case "open_url_in_player":
                        chrome.runtime.sendMessage({
                            type: "open_url_in_player",
                            url: data.url,
                        });
                        break;
                    default:
                    // Ignore unknown messages.
                }
            }
        }
    });

    await sendMessageToPage({
        type: "load",
        config: {
            ...explicitOptions,
            autoplay: options.autostart ? "on" : "auto",
            unmuteOverlay: options.autostart ? "hidden" : "visible",
            splashScreen: !options.autostart,
        },
        publicPath: utils.runtime.getURL("/dist/"),
    });
})();
