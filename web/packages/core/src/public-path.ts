import { Config } from "./config";

/**
 * Attempt to discover the public path of the current Ruffle source. This can
 * be used to configure Webpack.
 *
 * We have several points of configuration for how the Ruffle public path can
 * be determined:
 *
 * 1. The public path can be specified on a per-source basis using the
 * RufflePlayer config, for example:
 * `window.RufflePlayer.config.public_paths.local = "/dist/";`
 * 2. A global public path can be specified for all sources, also in config.
 * `window.RufflePlayer.config.publicPath = "/dist/";`
 * 3. If there is absolutely no configuration that yields a public path then we
 * return the parent path of where this script is hosted, which should be
 * the correct default in most cases.
 *
 * @param config The `window.RufflePlayer.config` object.
 * @param source_name The name of the source.
 * @returns The public path for the given source.
 */
export function publicPath(config: Config, source_name: string): string {
    let path = "";
    if (
        config !== undefined &&
        config.public_paths !== undefined &&
        config.public_paths[source_name] !== undefined
    ) {
        path = config.public_paths[source_name];
    } else if (config !== undefined && config.publicPath !== undefined) {
        path = config.publicPath;
    } else if (
        document.currentScript !== undefined &&
        document.currentScript !== null &&
        "src" in document.currentScript
    ) {
        // Default to the directory where this script resides.
        try {
            path = new URL(".", document.currentScript.src).href;
        } catch (e) {
            console.warn("Unable to get currentScript URL");
        }
    }

    // Webpack expects the paths to end with a /.
    if (path !== "" && !path.endsWith("/")) {
        path += "/";
    }

    return path;
}
