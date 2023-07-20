import * as utils from "./utils";
import { isSwf as isSwfCore } from "ruffle-core";

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
            redirectUrl: `${baseUrl}?url=${encodeURIComponent(details.url)}`,
        };
    }
    return undefined;
}

function enable() {
    (chrome || browser).webRequest.onHeadersReceived.addListener(
        onHeadersReceived,
        {
            urls: ["<all_urls>"],
            types: ["main_frame", "sub_frame"],
        },
        ["blocking", "responseHeaders"],
    );
}

function disable() {
    (chrome || browser).webRequest.onHeadersReceived.removeListener(
        onHeadersReceived,
    );
}

(async () => {
    const { ruffleEnable } = await utils.getOptions();

    if (ruffleEnable) {
        enable();
    }

    utils.storage.onChanged.addListener((changes, namespace) => {
        if (namespace === "sync" && "ruffleEnable" in changes) {
            if (changes["ruffleEnable"]!.newValue) {
                enable();
            } else {
                disable();
            }
        }
    });
})();
