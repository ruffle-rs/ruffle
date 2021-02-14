const { getSyncStorage, getExtensionUrl } = require("./util");

function injectScriptRaw(src) {
    const script = document.createElement("script");
    script.textContent = src;
    (document.head || document.documentElement).append(script);
}

function injectScriptURL(url) {
    const script = document.createElement("script");
    script.src = url;
    (document.head || document.documentElement).append(script);
}

const isEnabled = true;
if (isEnabled) {
    injectScriptRaw(require("./pluginPolyfill"));
    const path = "dist/ruffle.js";
    const url = getExtensionUrl(path);
    injectScriptRaw(`const ruffleRuntimePath = ${JSON.stringify(url.replace(path, ""))}`);
    injectScriptURL(url);
}
