/**
 * Conditional ruffle loader
 */

import {
    bulkMemory,
    simd,
    saturatedFloatToInt,
    signExtensions,
} from "wasm-feature-detect";
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

    // NOTE: Keep this list in sync with $RUSTFLAGS in the CI build config!
    const extensionsSupported: boolean = (
        await Promise.all([
            bulkMemory(),
            simd(),
            saturatedFloatToInt(),
            signExtensions(),
        ])
    ).every(Boolean);

    if (!extensionsSupported) {
        console.log(
            "Some WebAssembly extensions are NOT available, falling back to the vanilla WebAssembly module"
        );
    }

    __webpack_public_path__ = publicPath(config);

    // Note: The argument passed to import() has to be a simple string literal,
    // otherwise some bundler will get confused and won't include the module?
    const { default: init, Ruffle } = await (extensionsSupported
        ? import("../pkg/ruffle_web-wasm_extensions")
        : import("../pkg/ruffle_web"));

    await init();

    return Ruffle;
}

type Ruffle =
    | typeof import("../pkg/ruffle_web")["Ruffle"]
    | typeof import("../pkg/ruffle_web-wasm_extensions")["Ruffle"];

let lastLoaded: Promise<Ruffle> | null = null;

/**
 * Obtain an instance of `Ruffle`.
 *
 * This function returns a promise which yields `Ruffle` asynchronously.
 *
 * @param config The `window.RufflePlayer.config` object.
 * @returns A ruffle constructor that may be used to create new Ruffle
 * instances.
 */
export function loadRuffle(config: Config): Promise<Ruffle> {
    if (lastLoaded === null) {
        lastLoaded = fetchRuffle(config);
    }

    return lastLoaded;
}
