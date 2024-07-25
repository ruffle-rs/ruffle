import type { Options } from "./common";
import { DEFAULT_CONFIG as CORE_DEFAULT_CONFIG } from "ruffle-core";
import { SUPPORTED_PROTOCOLS } from "ruffle-core/dist/internal/constants";

const DEFAULT_OPTIONS: Required<Options> = {
    ...CORE_DEFAULT_CONFIG,
    ruffleEnable: true,
    ignoreOptout: false,
    autostart: false,
    showReloadButton: false,
    swfTakeover: true,
};

// TODO: Once https://crbug.com/798169 is addressed, just use browser.
// We have to wait until whatever version of Chromium supports that
// is old enough to be the oldest version we want to support.

export let i18n: typeof browser.i18n | typeof chrome.i18n;

type ScriptingType = (typeof browser.scripting | typeof chrome.scripting) & {
    ExecutionWorld: {
        MAIN: string | undefined;
        ISOLATED: string;
    };
};

export let scripting: ScriptingType;

export let storage: typeof browser.storage | typeof chrome.storage;

export let tabs: typeof browser.tabs | typeof chrome.tabs;

export let runtime: typeof browser.runtime | typeof chrome.runtime;

export let permissions: typeof browser.permissions | typeof chrome.permissions;

export let declarativeNetRequest:
    | typeof browser.declarativeNetRequest
    | typeof chrome.declarativeNetRequest;

function promisify<T>(
    func: (callback: (result: T) => void) => void,
): Promise<T> {
    return new Promise((resolve, reject) => {
        func((result) => {
            const error = chrome.runtime.lastError;
            if (error) {
                reject(error);
            } else {
                resolve(result);
            }
        });
    });
}

if (typeof browser !== "undefined") {
    i18n = browser.i18n;
    scripting = browser.scripting as ScriptingType;
    storage = browser.storage;
    tabs = browser.tabs;
    runtime = browser.runtime;
    permissions = browser.permissions;
    declarativeNetRequest = browser.declarativeNetRequest;
} else if (typeof chrome !== "undefined") {
    i18n = chrome.i18n;
    scripting = chrome.scripting as ScriptingType;
    storage = chrome.storage;
    tabs = chrome.tabs;
    runtime = chrome.runtime;
    permissions = chrome.permissions;
    declarativeNetRequest = chrome.declarativeNetRequest;
} else {
    throw new Error("Extension API not found.");
}
export const openOptionsPage: () => Promise<void> = () =>
    runtime.openOptionsPage();
export const openPlayerPage: () => Promise<void> = () =>
    promisify((cb: () => void) => tabs.create({ url: "/player.html" }, cb));
export const openOnboardPage: () => Promise<void> = () =>
    promisify((cb: () => void) => tabs.create({ url: "/onboard.html" }, cb));

export async function getOptions(): Promise<Options> {
    const options = await storage.sync.get();

    // Copy over default options if they don't exist yet.
    return { ...DEFAULT_OPTIONS, ...options };
}

/**
 * Gets the options that are explicitly different from the defaults.
 *
 * In the future we should just not store options we don't want to set.
 */
export async function getExplicitOptions(): Promise<Options> {
    const options = await getOptions();
    const defaultOptions = DEFAULT_OPTIONS;
    for (const key in defaultOptions) {
        // @ts-expect-error: Element implicitly has an any type
        if (key in options && defaultOptions[key] === options[key]) {
            // @ts-expect-error: Element implicitly has an any type
            // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
            delete options[key];
        }
    }

    return options;
}

/**
 * Returns whether the given URL is a URL that Ruffle can open.
 * @param url The given URL to be tested whether it can be opened.
 * @return Whether the given URL is a URL that Ruffle can open.
 */
export function supportedURL(url: URL | undefined): boolean {
    if (url) {
        return SUPPORTED_PROTOCOLS.includes(url.protocol);
    } else {
        return false;
    }
}

/**
 * Resolves a given string to a URL if possible.
 * If the protocol is missing and the string is otherwise a valid web URL, https:// is inserted if the
 * server supports https, otherwise http.
 * If the protocol is missing and the string is otherwise a valid file URL, file:/// is inserted.
 * If the string can't be resolved to a URL, null is returned.
 * @param enteredUrl The string that should be resolved to a URL.
 * @return The resolved URL object.
 */
export async function resolveSwfUrl(enteredUrl: string): Promise<URL | null> {
    // TODO: Use canParse in the future when it doesn't break browser compatibility
    // If the URL is (very likely) a file URL with missing file protocol, we return it as file URL
    // Must be the first test as the URL constructor accepts C:\… as URL with protocol C
    if (enteredUrl.match(/^[A-Za-z]:\\|^[/~\\]/)) {
        try {
            return new URL("file:///" + enteredUrl);
        } catch {
            return null;
        }
    }

    try {
        return new URL(enteredUrl);
    } catch {
        // The protocol is missing

        // If the URL doesn't contain a dot, it can't be a valid web URL
        // The URL constructor doesn't check this if a protocol exists
        if (!enteredUrl.includes(".")) {
            return null;
        }

        try {
            // TODO: Make the loading animation appear before waiting for the server response
            // Only use http if https doesn't work and http works
            // (Otherwise, error logs for offline websites would always contain http)
            if (
                (await serverAvailable("https://" + enteredUrl, 200)) ||
                !(await serverAvailable("http://" + enteredUrl, 100))
            ) {
                return new URL("https://" + enteredUrl);
            } else {
                return new URL("http://" + enteredUrl);
            }
        } catch {
            return null;
        }
    }
}

/**
 * Tests and returns whether a server exists under a given URL.
 * @param url The URL that should be tested.
 * @param timeout The maximum number of milliseconds that should be waited for a response.
 * @return Whether a server exists under the given URL.
 */
async function serverAvailable(
    url: string | URL,
    timeout: number,
): Promise<boolean> {
    // Polyfill for older browsers
    AbortSignal.timeout ??= function timeout(milliseconds) {
        const controller = new AbortController();
        setTimeout(() => controller.abort(), milliseconds);
        return controller.signal;
    };

    try {
        await fetch(url, {
            signal: AbortSignal.timeout(timeout),
            mode: "no-cors",
        });
        return true;
    } catch {
        return false;
    }
}

export const hasAllUrlsPermission = async () => {
    const allPermissions = await permissions.getAll();
    return allPermissions.origins?.includes("<all_urls>") ?? false;
};

export async function hasHostPermissionForSpecifiedTab(
    origin: string | undefined,
) {
    try {
        return await permissions.contains({ origins: [origin!] });
    } catch {
        // If the URL is invalid, don't ask for permission
        return true;
    }
}

export async function hasHostPermissionForActiveTab() {
    const [activeTab] = await tabs.query({
        active: true,
        currentWindow: true,
    });

    return await hasHostPermissionForSpecifiedTab(activeTab?.url);
}
