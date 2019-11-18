/**
 * Attempt to discover the public path of the current Ruffle source. This can
 * be used to configure Webpack.
 * 
 * We have several points of configuration for how the Ruffle public path can
 * be determined:
 * 
 * 1. The public path can be specified on a per-source basis using the
 * RufflePlayer config, for example:
 * `window.RufflePlayer.config.public_paths.local = "/dist";`
 * 2. A global public path can be specified for all sources, also in config.
 * `window.RufflePlayer.config.public_path = "/dist";`
 * 3. If there is absolutely no configuration that yields a public path then we
 * return "", which will configure resources to be loaded relative to the
 * current page. This is liable to be useless outside of contrived scenarios.
 * 
 * @param {object} config The `window.RufflePlayer.config` object.
 * @param {string} source_name The name of the source.
 * @returns {string} A 
 */
export function public_path(config, source_name) {
    if (config !== undefined && config.public_paths !== undefined && config.public_paths[source_name] !== undefined) {
        return config.public_paths[source_name];
    }

    if (config !== undefined && config.public_path !== undefined) {
        return config.public_path;
    }
    
    return "";
}