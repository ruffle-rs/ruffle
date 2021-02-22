import { getSyncStorage, getExtensionUrl } from "./utils";

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

// TODO: read settings
const isEnabled = true;
if (isEnabled) {
    injectScriptRaw(require("./pluginPolyfill")); // TODO: use plugin-polyfill.ts
    injectScriptURL(getExtensionUrl("dist/ruffle.js"));
}
