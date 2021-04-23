import * as utils from "./utils";

type WebResponseHeadersDetails = chrome.webRequest.WebResponseHeadersDetails | browser.webRequest._OnHeadersReceivedDetails;

function isSwf(details: WebResponseHeadersDetails) {
    // TypeScript doesn't compile without this explicit type delaration.
    const headers: (chrome.webRequest.HttpHeader | browser.webRequest._HttpHeaders)[] = details.responseHeaders!;
    const typeHeader = headers.find(
        ({ name }) => name.toLowerCase() === "content-type"
    );
    if (!typeHeader) {
        return false;
    }

    const mime = typeHeader.value!
        .toLowerCase()
        .match(/^\s*(.*?)\s*(?:;.*)?$/)![1];

    // Some sites (e.g. swfchan.net) might (wrongly?) send octet-stream, so check file extension too.
    if (mime === "application/octet-stream") {
        const url = new URL(details.url);
        const extension = url.pathname.substring(url.pathname.lastIndexOf("."));
        return extension.toLowerCase() === ".swf";
    }

    return mime === "application/x-shockwave-flash";
}

function onHeadersReceived(details: WebResponseHeadersDetails) {
    if (isSwf(details)) {
        const baseUrl = utils.runtime.getURL("player.html");
        return { redirectUrl: `${baseUrl}?url=${details.url}` };
    }
}

function enable() {
    (chrome || browser).webRequest.onHeadersReceived.addListener(
        onHeadersReceived,
        {
            urls: ["<all_urls>"],
            types: ["main_frame"],
        },
        ["blocking", "responseHeaders"]
    );
}

function disable() {
    (chrome || browser).webRequest.onHeadersReceived.removeListener(
        onHeadersReceived
    );
}

(async () => {
    const { ruffleEnable } = await utils.getOptions(["ruffleEnable"]);

    if (ruffleEnable) {
        enable();
    }

    utils.storage.onChanged.addListener((changes: any, namespace: string) => {
        if (
            namespace === "sync" &&
            Object.prototype.hasOwnProperty.call(changes, "ruffleEnable")
        ) {
            if (changes.ruffleEnable.newValue) {
                enable();
            } else {
                disable();
            }
        }
    });
})();
