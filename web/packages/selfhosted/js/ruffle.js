// eslint-disable-next-line no-unused-vars
/* global __webpack_public_path__:writable */

import { installRuffle } from "ruffle-core";

let currentScriptURL = null;

try {
    if (
        document.currentScript !== undefined &&
        document.currentScript !== null &&
        "src" in document.currentScript &&
        document.currentScript.src !== ""
    ) {
        let src = document.currentScript.src;

        // CDNs allow omitting the filename. If it's omitted, append a slash to
        // prevent the last component from being dropped.
        if (!src.endsWith(".js") && !src.endsWith("/")) {
            src += "/";
        }

        currentScriptURL = new URL(".", src);
    }
} catch (e) {
    console.warn("Unable to get currentScript URL");
}

function publicPath(config) {
    // Default to the directory where this script resides.
    let path = currentScriptURL?.href ?? "";
    if (
        "publicPath" in config &&
        config.publicPath !== null &&
        config.publicPath !== undefined
    ) {
        path = config.publicPath;
    }

    // Webpack expects the paths to end with a slash.
    if (path !== "" && !path.endsWith("/")) {
        path += "/";
    }

    return path;
}

installRuffle("local", {
    onFirstLoad: () => {
        __webpack_public_path__ = publicPath(window.RufflePlayer?.config);
    },
});
