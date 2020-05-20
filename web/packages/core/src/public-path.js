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
 * `window.RufflePlayer.config.public_path = "/dist/";`
 * 3. If there is absolutely no configuration that yields a public path then we
 * return the parent path of where this script is hosted, which should be
 * the correct default in most cases.
 *
 * @param {object} config The `window.RufflePlayer.config` object.
 * @param {string} source_name The name of the source.
 * @returns {string} The public path for the given source.
 */
exports.public_path = function public_path(config, source_name) {
    let public_path = "";
    if (
        config !== undefined &&
        config.public_paths !== undefined &&
        config.public_paths[source_name] !== undefined
    ) {
        public_path = config.public_paths[source_name];
    } else if (config !== undefined && config.public_path !== undefined) {
        public_path = config.public_path;
    } else if (document.currentScript !== undefined) {
        // Default to the directory where this script resides.
        try {
            public_path = new URL(".", document.currentScript.src).href;
        } catch (e) {
            console.warn("Unable to get currentScript URL");
        }
    }

    // Webpack expects the paths to end with a /.
    if (public_path !== "" && !public_path.endsWith("/")) {
        public_path += "/";
    }

    return public_path;
};
