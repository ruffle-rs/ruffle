/**
 * Conditional ruffle loader
 */

import init, { Ruffle } from "../pkg/ruffle_web";
import { setPolyfillsOnLoad } from "./js-polyfills";
import { publicPath } from "./public-path";
import { Config } from "./config";

declare global {
    let __webpack_public_path__: string;
}

/**
 * Load ruffle from an automatically-detected location.
 *
 * This function returns a new instance of Ruffle and downloads it every time.
 * You should not use it directly; this module will memoize the resource
 * download.
 *
 * @param config The `window.RufflePlayer.config` object.
 * @returns A ruffle constructor that may be used to create new Ruffle
 * instances.
 */
async function fetchRuffle(config: Config): Promise<typeof Ruffle> {
    // Apply some pure JavaScript polyfills to prevent conflicts with external
    // libraries, if needed.
    setPolyfillsOnLoad();

    __webpack_public_path__ = publicPath(config);
    await init();

    return Ruffle;
}

let lastLoaded: Promise<typeof Ruffle> | null = null;

/**
 * Obtain an instance of `Ruffle`.
 *
 * This function returns a promise which yields `Ruffle` asynchronously.
 *
 * @param config The `window.RufflePlayer.config` object.
 * @returns A ruffle constructor that may be used to create new Ruffle
 * instances.
 */
export function loadRuffle(config: Config): Promise<typeof Ruffle> {
    if (lastLoaded == null) {
        lastLoaded = fetchRuffle(config);
    }

    return lastLoaded;
}
