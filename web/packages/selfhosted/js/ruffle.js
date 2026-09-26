import { Setup } from "ruffle-core";

let currentScriptURL = null;

try {
    if (
        document.currentScript instanceof HTMLScriptElement &&
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
    console.warn("Unable to get currentScript URL", e);
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

    // Resolve relative/root paths into fully qualified absolute URLs
    try {
        return new URL(path, window.location.href).href;
    } catch (e) {
        console.warn("Unable to resolve publicPath URL", e);
        return path;
    }
}

Setup.installRuffle("local", {
    onFirstLoad: () => {
        if (typeof window !== "undefined") {
            window.__ruffle_public_path__ = publicPath(
                window.RufflePlayer?.config,
            );
        }
    },
});
