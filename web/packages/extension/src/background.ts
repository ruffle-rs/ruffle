import * as utils from "./utils";
import { isSwf as isSwfCore } from "ruffle-core";

const RULE_SWF_URL = 1;

function isSwf(
    details:
        | chrome.webRequest.WebResponseHeadersDetails
        | browser.webRequest._OnHeadersReceivedDetails,
) {
    // TypeScript doesn't compile without this explicit type declaration.
    const headers: (
        | chrome.webRequest.HttpHeader
        | browser.webRequest._HttpHeaders
    )[] = details.responseHeaders!;
    const typeHeader = headers.find(
        ({ name }) => name.toLowerCase() === "content-type",
    );
    if (!typeHeader) {
        return false;
    }

    const mimeType = typeHeader
        .value!.toLowerCase()
        .match(/^\s*(.*?)\s*(?:;.*)?$/)![1]!;

    return isSwfCore(details.url, mimeType);
}

function onHeadersReceived(
    details:
        | chrome.webRequest.WebResponseHeadersDetails
        | browser.webRequest._OnHeadersReceivedDetails,
) {
    if (isSwf(details)) {
        const baseUrl = utils.runtime.getURL("player.html");
        return {
            redirectUrl: `${baseUrl}#${details.url}`,
        };
    }
    return undefined;
}

async function enable() {
    if (chrome?.declarativeNetRequest) {
        const playerPage = chrome.runtime.getURL("/player.html");
        const rules = [
            {
                id: RULE_SWF_URL,
                action: {
                    type: chrome.declarativeNetRequest.RuleActionType.REDIRECT,
                    redirect: { regexSubstitution: playerPage + "#\\0" },
                },
                condition: {
                    regexFilter: "^.*\\.swf(\\?.*|#.*|)$",
                    resourceTypes: [
                        chrome.declarativeNetRequest.ResourceType.MAIN_FRAME,
                    ],
                },
            },
        ];
        await chrome.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [RULE_SWF_URL],
            addRules: rules,
        });

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
    } else {
        (chrome || browser).webRequest.onHeadersReceived.addListener(
            onHeadersReceived,
            {
                urls: ["<all_urls>"],
                types: ["main_frame", "sub_frame"],
            },
            ["blocking", "responseHeaders"],
        );
    }
}

async function disable() {
    if (chrome?.declarativeNetRequest) {
        await chrome.declarativeNetRequest.updateDynamicRules({
            removeRuleIds: [RULE_SWF_URL],
        });
        await chrome.scripting.unregisterContentScripts({
            ids: ["plugin-polyfill"],
        });
    } else {
        (chrome || browser).webRequest.onHeadersReceived.removeListener(
            onHeadersReceived,
        );
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
