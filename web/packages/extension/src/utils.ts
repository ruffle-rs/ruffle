import type { Options } from "./common";
import { DEFAULT_CONFIG as CORE_DEFAULT_CONFIG } from "ruffle-core";

const DEFAULT_OPTIONS: Required<Options> = {
    ...CORE_DEFAULT_CONFIG,
    ruffleEnable: true,
    ignoreOptout: false,
    autostart: false,
};

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
} else if (typeof chrome !== "undefined") {
    i18n = chrome.i18n;
    scripting = chrome.scripting as ScriptingType;
    storage = chrome.storage;
    tabs = chrome.tabs;
    runtime = chrome.runtime;
    permissions = chrome.permissions;
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
            delete options[key];
        }
    }

    return options;
}

export const hasAllUrlsPermission = async () => {
    const allPermissions = await permissions.getAll();
    return allPermissions.origins?.includes("<all_urls>") ?? false;
};

export async function hasHostPermissionForActiveTab() {
    const [activeTab] = await tabs.query({
        active: true,
        currentWindow: true,
    });

    try {
        return activeTab?.url
            ? await permissions.contains({
                  origins: [activeTab.url],
              })
            : await hasAllUrlsPermission();
    } catch {
        // catch error that occurs for special urls like about:
        return false;
    }
}
