/* eslint @typescript-eslint/no-explicit-any: "off" */

/**
 * Conditional ruffle loader
 */

import { Ruffle } from "../pkg/ruffle_web";

import { setArrayPrototypeReduce } from "./js-polyfills";

/**
 * Load ruffle from an automatically-detected location.
 *
 * This function returns a new instance of Ruffle and downloads it every time.
 * You should not use it directly; this module will memoize the resource
 * download.
 *
 * @returns A ruffle constructor that may be used to create new Ruffle
 * instances.
 */
async function fetchRuffle(): Promise<{ new (...args: any[]): Ruffle }> {
    if (
        typeof Array.prototype.reduce !== "function" ||
        Array.prototype.reduce.toString().indexOf("[native code]") === -1
    ) {
        // Some external libraries override the `Array.prototype.reduce` method in a way
        // that causes Webpack to crash (#1507, #1865), so we need to override it again.
        setArrayPrototypeReduce();
    }

    try {
        // If ruffleRuntimePath is defined then we are executing inside the extension
        // closure. In that case, we configure our local Webpack instance.
        __webpack_public_path__ = ruffleRuntimePath + "dist/";
    } catch (e) {
        // Checking an undefined closure variable usually throws ReferenceError,
        // so we need to catch it here and continue onward.
        if (!(e instanceof ReferenceError)) {
            throw e;
        }
    }

    // We currently assume that if we are not executing inside the extension,
    // then we can use webpack to get Ruffle.
    const module = await import("../pkg/ruffle_web");
    return module.Ruffle;
}

let lastLoaded: Promise<{ new (...args: any[]): Ruffle }> | null = null;

/**
 * Obtain an instance of `Ruffle`.
 *
 * This function returns a promise which yields `Ruffle` asynchronously.
 *
 * @returns A ruffle constructor that may be used to create new Ruffle
 * instances.
 */
export function loadRuffle(): Promise<{ new (...args: any[]): Ruffle }> {
    if (lastLoaded == null) {
        lastLoaded = fetchRuffle();
    }

    return lastLoaded;
}
