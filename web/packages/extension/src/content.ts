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
        to: `ruffle_page${ID}`,
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
function injectScriptRaw(src: string) {
    const script = document.createElement("script");
    script.textContent = src;
    (document.head || document.documentElement).append(script);
    script.remove();
}

/**
 * Inject a script by URL to the main world.
 * @param {string} url - Script URL to inject.
 */
function injectScriptURL(url: string): Promise<void> {
    const script = document.createElement("script");
    const promise = new Promise<void>((resolve, reject) => {
        script.addEventListener("load", function () {
            resolve();
            this.remove();
        });
        script.addEventListener("error", (e) => reject(e));
    });
    script.charset = "utf-8";
    script.src = url;
    (document.head || document.documentElement).append(script);
    return promise;
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

    // We must run the plugin polyfill before any flash detection scripts.
    // Unfortunately, this might still be too late for some websites (issue #969).
    // NOTE: The script code injected here is the compiled form of
    // plugin-polyfill.ts. It is injected by tools/inject_plugin_polyfill.ts
    // which just search-and-replaces for this particular string.
    // On browsers which support ExecutionWorld MAIN this will be done earlier.
    if (
        navigator.wrappedJSObject &&
        navigator.wrappedJSObject.plugins.namedItem("Shockwave Flash")
            ?.filename !== "ruffle.js"
    ) {
        injectScriptRaw("%PLUGIN_POLYFILL_SOURCE%");
    }

    await injectScriptURL(utils.runtime.getURL(`dist/ruffle.js?id=${ID}`));

    window.addEventListener("message", (event) => {
        // We only accept messages from ourselves.
        if (event.source !== window || !event.data) {
            return;
        }

        const { to, index, data } = event.data;
        if (to === `ruffle_content${ID}`) {
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
    const autoStartOptions = options.autostart
        ? {
              autoplay: "on",
              unmuteOverlay: "hidden",
              splashScreen: false,
          }
        : {};
    await sendMessageToPage({
        type: "load",
        config: {
            ...explicitOptions,
            ...autoStartOptions,
        },
    });
})();
