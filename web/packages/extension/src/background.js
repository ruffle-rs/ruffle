function isSwf(details) {
    const typeHeader = details.responseHeaders.find(({name}) => name.toLowerCase() === "content-type");
    if (!typeHeader) {
        return false;
    }

    const mime = typeHeader.value.toLowerCase().match(/^\s*(.*?)\s*(?:;.*)?$/)[1];

    // Some sites (e.g. swfchan.net) might (wrongly?) send octet-stream, so check file extension too.
    if (mime === "application/octet-stream") {
        const url = new URL(details.url);
        const extension = url.pathname.substring(url.pathname.lastIndexOf("."));
        return extension.toLowerCase() === ".swf";
    }

    return mime === "application/x-shockwave-flash";
}

function onHeadersReceived(details) {
    if (!isSwf(details)) {
        return;
    }

    const baseUrl = chrome.runtime.getURL("player.html");
    return { redirectUrl: `${baseUrl}?url=${details.url}` };
}

// TODO: Support Firefox.
// TODO: Only if configured.
chrome.webRequest.onHeadersReceived.addListener(
    onHeadersReceived,
    {
        urls: ["<all_urls>"],
        types: ["main_frame"],
    },
    ["blocking", "responseHeaders"]
);

// TODO: chrome.webRequest.onHeadersReceived.removeListener(onHeadersReceived);
