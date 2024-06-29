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

import { PublicAPI, PublicAPILike } from "./public-api";
import { SourceAPI } from "./source-api";

/**
 * Join a source into the public API, if it doesn't already exist.
 *
 * @param prevRuffle The previous iteration of the Ruffle API.
 *
 * The `prevRuffle` param lists the previous object in the RufflePlayer
 * slot. We perform some checks to see if this is a Ruffle public API or a
 * conflicting object. If this is conflicting, then a new public API will
 * be constructed (see the constructor information for what happens to
 * `prevRuffle`).
 *
 * Note that Public API upgrades are deliberately not enabled in this
 * version of Ruffle, since there is no Public API to upgrade from.
 * @param sourceName The name of this particular
 * Ruffle source. Common convention is "local" for websites that bundle their own Ruffle,
 * "extension" for browser extensions, and something else for other use cases.
 *
 * If both parameters are provided they will be used to define a new Ruffle
 * source to register with the public API.
 * @returns The Ruffle Public API.
 */
export function installRuffle(
    prevRuffle?: PublicAPILike | null,
    sourceName?: string,
): PublicAPI {
    let publicAPI: PublicAPI;
    if (prevRuffle instanceof PublicAPI) {
        publicAPI = prevRuffle;
    } else {
        publicAPI = new PublicAPI(prevRuffle);
    }

    if (sourceName !== undefined) {
        publicAPI.registerSource(sourceName);

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

    return publicAPI;
}
