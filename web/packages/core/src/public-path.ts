import { BaseLoadOptions } from "./load-options";
import { currentScriptURL, isExtension } from "./current-script";

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
export function publicPath(config: BaseLoadOptions): string {
    // Default to the directory where this script resides.
    let path = currentScriptURL?.href ?? "";
    if (
        !isExtension &&
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
