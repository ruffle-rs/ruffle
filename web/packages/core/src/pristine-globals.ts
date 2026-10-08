import { isNativeFunction } from "./js-polyfills.js";

// Some pages replace built-in globals with broken implementations (#19758).
// Bundlers rewrite free references to the globals exported from here (e.g. via
// esbuild's `inject`), so that our code and our dependencies never use what
// the page has put in their place, and we never have to modify the page's =
// globals.
//
// We only fall back to a fresh realm when a global looks modified, instead of
// always using one. The extension runs this module at document_start in every
// frame of every page, and creating a hidden iframe each time would add a new
// realm and a DOM mutation per frame, which is a noticeable performance cost.
//
// Nothing in this module may reference these globals by their bare name,
// as those references would be rewritten to point back at this module.

type PristineGlobals = Pick<typeof globalThis, "Map">;

let freshGlobals: PristineGlobals | null = null;

/**
 * Returns unmodified globals from a fresh realm, created on first use.
 *
 * @returns Globals from a fresh realm
 */
function getFreshGlobals(): PristineGlobals {
    if (freshGlobals === null) {
        const iframe = document.createElement("iframe");
        iframe.style.display = "none";
        document.documentElement.append(iframe);
        const freshWindow =
            iframe.contentWindow as unknown as typeof globalThis;
        freshGlobals = {
            Map: freshWindow.Map,
        };
        iframe.remove();
    }
    return freshGlobals;
}

/**
 * Returns the given global from the current global scope if it's unmodified,
 * otherwise from a fresh realm.
 *
 * @param name Name of the global to retrieve
 * @param isPristine Whether the given value of the global is unmodified
 * @returns An unmodified value of the global
 */
function pristineGlobal<K extends keyof PristineGlobals>(
    name: K,
    isPristine: (value: PristineGlobals[K]) => boolean,
): PristineGlobals[K] {
    const value = globalThis[name];
    if (
        (value !== undefined && value !== null && isPristine(value)) ||
        // Without a document (e.g. in a worker), there's no fresh realm to use.
        typeof document === "undefined"
    ) {
        return value;
    }
    return getFreshGlobals()[name];
}

const PristineMap = pristineGlobal(
    "Map",
    (map) =>
        isNativeFunction(map) &&
        (["get", "set", "has", "delete", "clear", "forEach"] as const).every(
            (method) =>
                typeof map.prototype[method] === "function" &&
                isNativeFunction(map.prototype[method]),
        ),
);

export { PristineMap as Map };
