import { BaseLoadOptions } from "./public/config/index.js";
import { currentScriptURL, isExtension } from "./current-script.js";

/**
 * Attempt to discover the public path of the current Ruffle source. This is
 * used to locate Ruffle's other files, such as the .wasm module.
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

    // Relative URLs resolve against the last directory, so it needs a slash.
    if (path !== "" && !path.endsWith("/")) {
        path += "/";
    }

    return path;
}
