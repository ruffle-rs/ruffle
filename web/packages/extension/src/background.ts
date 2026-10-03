import {
    enableBrowserOnOutdatedChromium,
    getOptions,
    hasAllUrlsPermission,
    openOnboardPage,
} from "./utils";
import { isMessage } from "./messages";

/**
 * Returns whether the plugin polyfill content script is registered and whether
 * its JS file matches the expected variant.
 */
async function getPluginPolyfillRegistration(
    expectedScript?: string,
): Promise<{ registered: boolean; matches: boolean }> {
    const matchingScripts = await browser.scripting.getRegisteredContentScripts(
        {
            ids: ["plugin-polyfill"],
        },
    );

    if (matchingScripts?.length === 0) {
        return {
            registered: false,
            matches: false,
        };
    }

    // Content script IDs are unique, so there is at most one matching script.
    const script = matchingScripts[0];

    return {
        registered: true,
        matches: script?.js?.[0] === expectedScript,
    };
}

// Copied from https://github.com/w3c/webextensions/issues/638#issuecomment-2181124486
async function isHeaderConditionSupported() {
    let needCleanup: boolean;
    const ruleId = 4;
    try {
        // Throws synchronously if not supported.
        await browser.declarativeNetRequest.updateDynamicRules({
            addRules: [
                {
                    id: ruleId,
                    condition: { responseHeaders: [{ header: "whatever" }] },
                    action: {
                        type:
                            browser.declarativeNetRequest.RuleActionType
                                ?.ALLOW ?? "allow",
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
        await browser.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [ruleId],
            addRules: [
                {
                    id: ruleId,
                    condition: { responseHeaders: [] },
                    action: {
                        type:
                            browser.declarativeNetRequest.RuleActionType
                                ?.ALLOW ?? "allow",
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
            await browser.declarativeNetRequest.updateDynamicRules({
                removeRuleIds: [ruleId],
            });
        }
    }
}

async function enableSWFTakeover() {
    // Checks if the responseHeaders condition is supported and not behind a disabled flag.
    if (browser.declarativeNetRequest && (await isHeaderConditionSupported())) {
        const { ruffleEnable } = await getOptions();
        if (ruffleEnable) {
            const playerPage = browser.runtime.getURL("/player.html");
            const rules = [
                {
                    id: 1,
                    action: {
                        type:
                            browser.declarativeNetRequest.RuleActionType
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
                            browser.declarativeNetRequest.ResourceType
                                ?.MAIN_FRAME ?? "main_frame",
                        ],
                    },
                },
                {
                    id: 2,
                    action: {
                        type:
                            browser.declarativeNetRequest.RuleActionType
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
                            browser.declarativeNetRequest.ResourceType
                                ?.MAIN_FRAME ?? "main_frame",
                        ],
                    },
                },
                {
                    id: 3,
                    action: {
                        type:
                            browser.declarativeNetRequest.RuleActionType
                                ?.REDIRECT ?? "redirect",
                        redirect: { regexSubstitution: playerPage + "#\\0" },
                    },
                    condition: {
                        regexFilter:
                            "^.*:\\/\\/.*\\/.*\\.s(?:wf|pl)(\\?.*|#.*|)$",
                        excludedResponseHeaders: [{ header: "content-type" }],
                        resourceTypes: [
                            browser.declarativeNetRequest.ResourceType
                                ?.MAIN_FRAME ?? "main_frame",
                        ],
                    },
                },
            ];
            await browser.declarativeNetRequest.updateDynamicRules({
                removeRuleIds: [1, 2, 3],
                addRules: rules,
            });
        }
        browser.storage.sync.set({ responseHeadersUnsupported: false });
    } else {
        browser.storage.sync.set({ responseHeadersUnsupported: true });
    }
}

async function disableSWFTakeover() {
    if (browser.declarativeNetRequest && (await isHeaderConditionSupported())) {
        await browser.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [1, 2, 3],
        });
        browser.storage.sync.set({ responseHeadersUnsupported: false });
    } else {
        browser.storage.sync.set({ responseHeadersUnsupported: true });
    }
}

async function enable() {
    const { swfTakeover, ignoreOptout } = await getOptions();
    if (swfTakeover) {
        await enableSWFTakeover();
    }
    if (
        !browser.scripting ||
        (browser.scripting.ExecutionWorld &&
            !browser.scripting.ExecutionWorld.MAIN)
    ) {
        return;
    }
    const expectedScript = ignoreOptout
        ? "dist/pluginPolyfillIgnoreOptout.js"
        : "dist/pluginPolyfill.js";

    const { registered, matches } =
        await getPluginPolyfillRegistration(expectedScript);
    if (!matches) {
        if (registered) {
            await browser.scripting.unregisterContentScripts({
                ids: ["ruffle", "plugin-polyfill", "4399"],
            });
        }
        // Reuse the exclude_matches of dist/content.js in the manifest.
        const excludeMatches =
            browser.runtime.getManifest().content_scripts![0]!.exclude_matches!;
        await browser.scripting.registerContentScripts([
            {
                id: "ruffle",
                js: ["dist/ruffle.js"],
                persistAcrossSessions: true,
                matches: ["<all_urls>"],
                excludeMatches,
                runAt: "document_start",
                allFrames: true,
                world: "MAIN",
            },
            {
                id: "plugin-polyfill",
                js: [expectedScript],
                persistAcrossSessions: true,
                matches: ["<all_urls>"],
                excludeMatches,
                runAt: "document_start",
                allFrames: true,
                world: "MAIN",
            },
            {
                id: "4399",
                matches: [
                    "*://www.4399.com/flash/*",
                    "https://my.4399.com/*",
                    "https://news.4399.com/qiu/",
                    "http://sjsj.4399.com/",
                ],
                js: ["dist/siteContentScript4399.js"],
                world: "MAIN",
                runAt: "document_start",
            },
        ]);
    }
}

async function disable() {
    if (
        !browser.scripting ||
        (browser.scripting.ExecutionWorld &&
            !browser.scripting.ExecutionWorld.MAIN)
    ) {
        return;
    }
    const { registered } = await getPluginPolyfillRegistration();
    if (registered) {
        await browser.scripting.unregisterContentScripts({
            ids: ["ruffle", "plugin-polyfill", "4399"],
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
        await browser.storage.sync.set({
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
            browser.tabs.create({
                url: browser.runtime.getURL(`player.html#${request.url}`),
            });
        }
    }
}

(async () => {
    enableBrowserOnOutdatedChromium();
    const { ruffleEnable } = await getOptions();
    if (ruffleEnable) {
        await enable();
    }
})();

// Listeners must be registered synchronously at the top level,
// otherwise they won't be called in time when the service worker wakes up
if (browser?.runtime && !browser.runtime.onMessage.hasListener(onMessage)) {
    browser.runtime.onMessage.addListener(onMessage);
}

browser.storage.onChanged.addListener(async (changes, namespace) => {
    if (namespace === "sync" && "ruffleEnable" in changes) {
        if (changes["ruffleEnable"]!.newValue) {
            await enable();
        } else {
            await disable();
        }
    }
    if (namespace === "sync" && "ignoreOptout" in changes) {
        const { ruffleEnable } = await getOptions();
        if (ruffleEnable) {
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
        details.reason === browser.runtime.OnInstalledReason.INSTALL &&
        !(await hasAllUrlsPermission())
    ) {
        await openOnboardPage();
    }
}

browser.runtime.onInstalled.addListener(handleInstalled);
browser.permissions.onAdded.addListener(onAdded);
