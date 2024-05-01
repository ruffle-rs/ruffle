import * as utils from "./utils";
import { isMessage } from "./messages";

async function contentScriptRegistered() {
    const matchingScripts = await chrome.scripting.getRegisteredContentScripts({
        ids: ["plugin-polyfill"],
    });
    return matchingScripts.length > 0;
}

async function enable() {
    if (chrome?.scripting && !(await contentScriptRegistered())) {
        await chrome.scripting.registerContentScripts([
            {
                id: "plugin-polyfill",
                js: ["dist/pluginPolyfill.js"],
                persistAcrossSessions: true,
                matches: ["<all_urls>"],
                excludeMatches: [
                    "https://sso.godaddy.com/*",
                    "https://authentication.td.com/*",
                    "https://*.twitch.tv/*",
                    "https://www.tuxedocomputers.com/*",
                    "https://*.taobao.com/*",
                    "https://*.time4learning.com/*",
                    "https://*.edgenuity.com/*",
                ],
                runAt: "document_start",
                world: "MAIN",
            },
        ]);
    }
}

async function disable() {
    if (chrome?.scripting && (await contentScriptRegistered())) {
        await chrome.scripting.unregisterContentScripts({
            ids: ["plugin-polyfill"],
        });
    }
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
})();

// Listeners must be registered synchronously at the top level,
// otherwise they won't be called in time when the service worker wakes up
if (chrome?.runtime && !chrome.runtime.onMessage.hasListener(onMessage)) {
    chrome.runtime.onMessage.addListener(onMessage);
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
