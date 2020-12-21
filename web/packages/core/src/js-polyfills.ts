/* eslint @typescript-eslint/no-explicit-any: "off" */

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
export function setArrayPrototypeReduce(): any {
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
