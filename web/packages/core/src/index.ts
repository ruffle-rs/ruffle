/**
 * This is the public API of the web version of Ruffle.
 *
 * Types should only be exported here if they are intended to be part of that public API, not internal.
 */

export * from "./polyfills";
export * from "./public-api";
export * from "./ruffle-player";
export * from "./version";
export * from "./version-range";
export * from "./config";
export * from "./load-options";
export * from "./build-info";
export * from "./swf-utils";
export * from "./movie-metadata";

import { PublicAPI } from "./public-api";
import { SourceAPI } from "./source-api";

/**
 * Install this version of Ruffle into the current page.
 *
 * Multiple (or zero) versions of Ruffle may be installed at the same time,
 * and you should use `window.RufflePlayer.newest()` or similar to access the appropriate
 * installation at time of use.
 *
 * @param sourceName The name of this particular
 * Ruffle source. Common convention is "local" for websites that bundle their own Ruffle,
 * "extension" for browser extensions, and something else for other use cases.
 * Names are unique, and last-installed will replace earlier installations with the same name,
 * regardless of what those installations are or which version they represent.
 */
export function installRuffle(sourceName: string): void {
    let publicAPI: PublicAPI;
    if (window.RufflePlayer instanceof PublicAPI) {
        publicAPI = window.RufflePlayer;
    } else {
        publicAPI = new PublicAPI(window.RufflePlayer);
        window.RufflePlayer = publicAPI;
    }

    publicAPI.sources[sourceName] = SourceAPI;

    // Install the faux plugin detection immediately.
    // This is necessary because scripts such as SWFObject check for the
    // Flash Player immediately when they load.
    // TODO: Maybe there's a better place for this.
    const polyfills =
        "polyfills" in publicAPI.config ? publicAPI.config.polyfills : true;
    if (polyfills !== false) {
        SourceAPI.pluginPolyfill();
    }
}
