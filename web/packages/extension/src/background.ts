import * as utils from "./utils";
import { isMessage } from "./messages";

async function enable() {
    if (chrome?.scripting) {
        await chrome.scripting.registerContentScripts([
            {
                id: "plugin-polyfill",
                js: ["dist/pluginPolyfill.js"],
                persistAcrossSessions: false,
                matches: ["<all_urls>"],
                excludeMatches: [
                    "https://sso.godaddy.com/*",
                    "https://authentication.td.com/*",
                    "https://*.twitch.tv/*",
                    "https://www.tuxedocomputers.com/*",
                    "https://*.taobao.com/*",
                ],
                runAt: "document_start",
                world: "MAIN",
            },
        ]);
    }

    chrome.runtime.onMessage.addListener(onMessage);
}

async function disable() {
    if (chrome?.scripting) {
        await chrome.scripting.unregisterContentScripts({
            ids: ["plugin-polyfill"],
        });
    }

    chrome.runtime.onMessage.removeListener(onMessage);
}

function onMessage(
    request: unknown,
    _sender: chrome.runtime.MessageSender,
    _sendResponse: (response: unknown) => void,
): void {
    if (isMessage(request)) {
        if (request.type === "open_url_in_player") {
            chrome.tabs.create({
                url: utils.runtime.getURL(`player.html#${request.url}`),
            });
        }
    }
}

(async () => {
    const { ruffleEnable } = await utils.getOptions();

    if (ruffleEnable) {
        await enable();
    }

    utils.storage.onChanged.addListener(async (changes, namespace) => {
        if (namespace === "sync" && "ruffleEnable" in changes) {
            if (changes["ruffleEnable"]!.newValue) {
                await enable();
            } else {
                await disable();
            }
        }
    });
})();
