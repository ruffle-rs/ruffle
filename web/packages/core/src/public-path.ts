import { Config } from "./config";

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
    let path = "";
    if (config !== undefined && config.publicPath !== undefined) {
        path = config.publicPath;
    } else if (
        document.currentScript !== undefined &&
        document.currentScript !== null &&
        "src" in document.currentScript &&
        document.currentScript.src !== ""
    ) {
        // Default to the directory where this script resides.
        try {
            path = new URL(".", document.currentScript.src).href;
        } catch (e) {
            console.warn("Unable to get currentScript URL");
        }
    }

    // Webpack expects the paths to end with a slash.
    if (path !== "" && !path.endsWith("/")) {
        path += "/";
    }

    return path;
}
