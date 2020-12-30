/* eslint @typescript-eslint/no-explicit-any: "off" */
/* eslint @typescript-eslint/ban-types: "off" */

declare global {
    interface Window {
        Prototype?: any;
    }
}

/**
 * Polyfills the `Array.prototype.reduce` method.
 *
 * Production steps of ECMA-262, Edition 5, 15.4.4.21
 * Reference: https://es5.github.io/#x15.4.4.21
 * https://tc39.github.io/ecma262/#sec-array.prototype.reduce
 *
 */
function polyfillArrayPrototypeReduce(): any {
    Object.defineProperty(Array.prototype, "reduce", {
        value: function (...args: any) {
            if (
                args.length === 0 &&
                window.Prototype &&
                window.Prototype.Version &&
                window.Prototype.Version < "1.6.1"
            ) {
                // Off-spec: compatibility with prototype.js
                return this.length > 1 ? this : this[0];
            }

            const callback = args[0];
            if (this === null) {
                throw new TypeError(
                    "Array.prototype.reduce called on null or undefined"
                );
            }
            if (typeof callback !== "function") {
                throw new TypeError(`${callback} is not a function`);
            }

            const o = Object(this);
            const len = o.length >>> 0;
            let k = 0;
            let value;

            if (args.length >= 2) {
                value = args[1];
            } else {
                while (k < len && !(k in o)) {
                    k++;
                }
                if (k >= len) {
                    throw new TypeError(
                        "Reduce of empty array with no initial value"
                    );
                }
                value = o[k++];
            }

            while (k < len) {
                if (k in o) {
                    value = callback(value, o[k], k, o);
                }
                k++;
            }

            return value;
        },
    });
}

/**
 * Polyfills the `Window` function.
 *
 */
function polyfillWindow(): void {
    if (
        typeof window.constructor !== "function" ||
        !isNativeFunction(window.constructor)
    ) {
        // Don't polyfill `Window` if `window.constructor` has been overridden.
        return;
    }
    // @ts-expect-error: `Function not assignable to { new (): Window; prototype: Window; }`
    window.Window = window.constructor;
}

/**
 * Determines whether a function is native or not.
 *
 * @param func The function to test.
 * @returns True if the function hasn't been overridden.
 */
function isNativeFunction(func: Function): boolean {
    const val =
        typeof Function.prototype.toString === "function"
            ? Function.prototype.toString()
            : null;
    if (typeof val === "string" && val.indexOf("[native code]") >= 0) {
        return (
            Function.prototype.toString.call(func).indexOf("[native code]") >= 0
        );
    }
    return false;
}

/**
 * Checks and applies the polyfills to the current window, if needed.
 *
 */
export function setPolyfillsOnLoad(): void {
    if (
        typeof Array.prototype.reduce !== "function" ||
        !isNativeFunction(Array.prototype.reduce)
    ) {
        // Some external libraries override the `Array.prototype.reduce` method in a way
        // that causes Webpack to crash (#1507, #1865), so we need to override it again.
        polyfillArrayPrototypeReduce();
    }
    if (typeof Window !== "function" || !isNativeFunction(Window)) {
        // Overriding the native `Window` function causes issues in wasm-bindgen, as a
        // code like `window instanceof Window` will no longer work.
        polyfillWindow();
    }
}
