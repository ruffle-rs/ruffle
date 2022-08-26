import type { Config } from "./config";

// This must be in global scope because `document.currentScript`
// works only while the script is initially being processed.
let currentScriptURL = "";
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

        currentScriptURL = new URL(".", src).href;
    }
} catch (e) {
    console.warn("Unable to get currentScript URL");
}

/**
 * Attempt to discover the public path of the current Ruffle source. This can
 * be used to configure Webpack.
 *
 * A global public path can be specified for all sources using the RufflePlayer
 * config:
 *
 * ```js
 * window.RufflePlayer.config.publicPath = "/dist/";
 * ```
 *
 * If no such config is specified, then the parent path of where this script is
 * hosted is assumed, which should be the correct default in most cases.
 *
 * @param config The `window.RufflePlayer.config` object.
 * @returns The public path for the given source.
 */
export function publicPath(config: Config): string {
    // Default to the directory where this script resides.
    let path = currentScriptURL;
    if (config !== undefined && config.publicPath !== undefined) {
        path = config.publicPath;
    }

    // Webpack expects the paths to end with a slash.
    if (path !== "" && !path.endsWith("/")) {
        path += "/";
    }

    return path;
}
