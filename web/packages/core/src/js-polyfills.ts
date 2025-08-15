declare global {
    interface Window {
        Prototype?: {
            Version?: string;
        };
        Map: typeof Map;
    }
}

/**
 * Polyfills the `Array.prototype.reduce` method.
 *
 * Production steps of ECMA-262, Edition 5, 15.4.4.21
 * Reference: https://es5.github.io/#x15.4.4.21
 * https://tc39.github.io/ecma262/#sec-array.prototype.reduce
 */
function polyfillArrayPrototypeReduce() {
    Object.defineProperty(Array.prototype, "reduce", {
        value(...args: unknown[]) {
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
                    "Array.prototype.reduce called on null or undefined",
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
                        "Reduce of empty array with no initial value",
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
 * Polyfills the `Reflect` object and members.
 *
 * This is a partial implementation, just enough to match our needs.
 */
function tryPolyfillReflect(): void {
    if (window.Reflect === undefined || window.Reflect === null) {
        // @ts-expect-error: {} indeed doesn't implement Reflect's interface.
        window.Reflect = {};
    }
    if (typeof Reflect.get !== "function") {
        Object.defineProperty(Reflect, "get", {
            value<T>(target: T, key: keyof T) {
                return target[key];
            },
        });
    }
    if (typeof Reflect.set !== "function") {
        Object.defineProperty(Reflect, "set", {
            value<T>(target: T, key: keyof T, value: T[keyof T]) {
                target[key] = value;
            },
        });
    }
    if (typeof Reflect.has !== "function") {
        Object.defineProperty(Reflect, "has", {
            value<T>(target: T, key: keyof T) {
                // @ts-expect-error: Type 'T' is not assignable to type 'object'.
                return key in target;
            },
        });
    }
    if (typeof Reflect.ownKeys !== "function") {
        Object.defineProperty(Reflect, "ownKeys", {
            value<T>(target: T) {
                return [
                    ...Object.getOwnPropertyNames(target),
                    ...Object.getOwnPropertySymbols(target),
                ];
            },
        });
    }
}

/**
 * Replaces a `Map` object missing standard methods with an unchanged `Map` object from a fresh global.
 *
 * @returns The custom `Map` object that exists on the page, or undefined if the page uses the standard `Map`.
 */
export function resetCustomMap(): typeof Map | undefined {
    if (typeof Map.prototype.set !== "function") {
        const currentMap = Map;
        const iframe = document.createElement("iframe");
        iframe.style.display = "none";
        document.documentElement.append(iframe);
        // eslint-disable-next-line no-global-assign
        Map = iframe.contentWindow!.Map;
        iframe.remove();
        return currentMap;
    }
    return undefined;
}

/**
 * Restores a custom map object to the global namespace if one was defined, as in https://github.com/ruffle-rs/ruffle/discussions/19758.
 *
 * @param customMap The custom `Map` object that existed on the page, or undefined if the page used the standard `Map`.
 */
export function restoreCustomMap(customMap: typeof Map | undefined) {
    if (customMap) {
        // eslint-disable-next-line no-global-assign
        Map = customMap;
    }
}

/**
 * Determines whether a function is native or not.
 *
 * @param func The function to test.
 * @returns True if the function hasn't been overridden.
 */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
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
    // Some pages override the native `Reflect` object, which causes various issues:
    // 1- wasm-bindgen's stdlib may crash (#3173).
    // 2- FlashVars may be ignored (#8537).
    tryPolyfillReflect();
}
