import * as utils from "./utils";
import { isMessage } from "./messages";

async function contentScriptRegistered() {
    const matchingScripts = await utils.scripting.getRegisteredContentScripts({
        ids: ["plugin-polyfill"],
    });
    return matchingScripts?.length > 0;
}

// Copied from https://github.com/w3c/webextensions/issues/638#issuecomment-2181124486
async function isHeaderConditionSupported() {
    let needCleanup = false;
    const ruleId = 4;
    try {
        // Throws synchronously if not supported.
        await utils.declarativeNetRequest.updateDynamicRules({
            addRules: [
                {
                    id: ruleId,
                    condition: { responseHeaders: [{ header: "whatever" }] },
                    action: {
                        type:
                            chrome.declarativeNetRequest.RuleActionType
                                ?.BLOCK ?? "block",
                    },
                },
            ],
        });
        needCleanup = true;
    } catch {
        return false; // responseHeaders condition not supported.
    }
    // Chrome may recognize the properties but have the implementation behind a flag.
    // When the implementation is disabled, validation is skipped too.
    try {
        await utils.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [ruleId],
            addRules: [
                {
                    id: ruleId,
                    condition: { responseHeaders: [] },
                    action: {
                        type:
                            chrome.declarativeNetRequest.RuleActionType
                                ?.BLOCK ?? "block",
                    },
                },
            ],
        });
        needCleanup = true;
        return false; // Validation skipped = feature disabled.
    } catch {
        return true; // Validation worked = feature enabled.
    } finally {
        if (needCleanup) {
            await utils.declarativeNetRequest.updateDynamicRules({
                removeRuleIds: [ruleId],
            });
        }
    }
}

async function enableSWFTakeover() {
    // Checks if the responseHeaders condition is supported and not behind a disabled flag.
    if (utils.declarativeNetRequest && (await isHeaderConditionSupported())) {
        const { ruffleEnable } = await utils.getOptions();
        if (ruffleEnable) {
            const playerPage = utils.runtime.getURL("/player.html");
            const rules = [
                {
                    id: 1,
                    action: {
                        type:
                            chrome.declarativeNetRequest.RuleActionType
                                ?.REDIRECT ?? "redirect",
                        redirect: { regexSubstitution: playerPage + "#\\0" },
                    },
                    condition: {
                        regexFilter: ".*",
                        responseHeaders: [
                            {
                                header: "content-type",
                                values: [
                                    "application/x-shockwave-flash",
                                    "application/futuresplash",
                                    "application/x-shockwave-flash2-preview",
                                    "application/vnd.adobe.flash.movie",
                                ],
                            },
                        ],
                        resourceTypes: [
                            chrome.declarativeNetRequest.ResourceType
                                ?.MAIN_FRAME ?? "main_frame",
                        ],
                    },
                },
                {
                    id: 2,
                    action: {
                        type:
                            chrome.declarativeNetRequest.RuleActionType
                                ?.REDIRECT ?? "redirect",
                        redirect: { regexSubstitution: playerPage + "#\\0" },
                    },
                    condition: {
                        regexFilter:
                            "^.*:\\/\\/.*\\/.*\\.s(?:wf|pl)(\\?.*|#.*|)$",
                        responseHeaders: [
                            {
                                header: "content-type",
                                values: [
                                    "application/octet-stream",
                                    "application/binary-stream",
                                    "",
                                ],
                            },
                        ],
                        resourceTypes: [
                            chrome.declarativeNetRequest.ResourceType
                                ?.MAIN_FRAME ?? "main_frame",
                        ],
                    },
                },
                {
                    id: 3,
                    action: {
                        type:
                            chrome.declarativeNetRequest.RuleActionType
                                ?.REDIRECT ?? "redirect",
                        redirect: { regexSubstitution: playerPage + "#\\0" },
                    },
                    condition: {
                        regexFilter:
                            "^.*:\\/\\/.*\\/.*\\.s(?:wf|pl)(\\?.*|#.*|)$",
                        excludedResponseHeaders: [{ header: "content-type" }],
                        resourceTypes: [
                            chrome.declarativeNetRequest.ResourceType
                                ?.MAIN_FRAME ?? "main_frame",
                        ],
                    },
                },
            ];
            await utils.declarativeNetRequest.updateDynamicRules({
                removeRuleIds: [1, 2, 3],
                addRules: rules,
            });
        }
    } else {
        utils.storage.sync.set({ responseHeadersUnsupported: true });
    }
}

async function disableSWFTakeover() {
    if (utils.declarativeNetRequest && (await isHeaderConditionSupported())) {
        await utils.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [1, 2, 3],
        });
    } else {
        utils.storage.sync.set({ responseHeadersUnsupported: true });
    }
}

async function enable() {
    const { swfTakeover } = await utils.getOptions();
    if (swfTakeover) {
        await enableSWFTakeover();
    }
    if (
        !utils.scripting ||
        (utils.scripting.ExecutionWorld && !utils.scripting.ExecutionWorld.MAIN)
    ) {
        return;
    }
    if (!(await contentScriptRegistered())) {
        await utils.scripting.registerContentScripts([
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
            {
                id: "4399",
                matches: ["https://www.4399.com/flash/*"],
                js: ["dist/siteContentScript4399.js"],
                world: "MAIN",
                runAt: "document_start",
            },
        ]);
    }
}

async function disable() {
    if (
        !utils.scripting ||
        (utils.scripting.ExecutionWorld && !utils.scripting.ExecutionWorld.MAIN)
    ) {
        return;
    }
    if (await contentScriptRegistered()) {
        await utils.scripting.unregisterContentScripts({
            ids: ["plugin-polyfill", "4399"],
        });
    }
    await disableSWFTakeover();
}

async function onAdded(permissions: chrome.permissions.Permissions) {
    if (
        permissions.origins &&
        permissions.origins.length >= 1 &&
        permissions.origins[0] !== "<all_urls>"
    ) {
        await utils.storage.sync.set({
            ["showReloadButton"]: true,
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
    if (namespace === "sync" && "swfTakeover" in changes) {
        if (changes["swfTakeover"]!.newValue) {
            await enableSWFTakeover();
        } else {
            await disableSWFTakeover();
        }
    }
});

async function handleInstalled(details: chrome.runtime.InstalledDetails) {
    if (
        details.reason === chrome.runtime.OnInstalledReason.INSTALL &&
        !(await utils.hasAllUrlsPermission())
    ) {
        await utils.openOnboardPage();
    }
}

chrome.runtime.onInstalled.addListener(handleInstalled);
utils.permissions.onAdded.addListener(onAdded);
