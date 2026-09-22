import type { Options } from "./common";
import { Config } from "ruffle-core";

const DEFAULT_OPTIONS: Required<Options> = {
    ...Config.DEFAULT_CONFIG,
    ruffleEnable: true,
    ignoreOptout: false,
    autostart: false,
    showReloadButton: false,
    swfTakeover: true,
};

// TODO: Once Chromium 148 is old enough to be the
// oldest version we want to support, remove this.
export function enableBrowserOnOutdatedChromium() {
    if (!globalThis.browser) {
        globalThis.browser = chrome;
    }
}

function promisify<T>(
    func: (callback: (result: T) => void) => void,
): Promise<T> {
    return new Promise((resolve, reject) => {
        func((result) => {
            const error = browser.runtime.lastError;
            if (error) {
                reject(error);
            } else {
                resolve(result);
            }
        });
    });
}

export const openOptionsPage: () => Promise<void> = () =>
    browser.runtime.openOptionsPage();
export const openPlayerPage: () => Promise<void> = () =>
    promisify((cb: () => void) =>
        browser.tabs.create({ url: "/player.html" }, cb),
    );
export const openOnboardPage: () => Promise<void> = () =>
    promisify((cb: () => void) =>
        browser.tabs.create({ url: "/onboard.html" }, cb),
    );

export async function getOptions(): Promise<Options> {
    const options = await browser.storage.sync.get();

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
    // This value is specific to the internal extension pages, and is always "default"
    if ("responseHeadersUnsupported" in options) {
        delete options["responseHeadersUnsupported"];
    }

    return options;
}

export const hasAllUrlsPermission = async () => {
    const allPermissions = await browser.permissions.getAll();
    return allPermissions.origins?.includes("<all_urls>") ?? false;
};

export async function hasHostPermissionForSpecifiedTab(
    origin: string | undefined,
) {
    try {
        return origin
            ? await browser.permissions.contains({
                  origins: [origin],
              })
            : await hasAllUrlsPermission();
    } catch {
        // catch error that occurs for special urls like about:
        return false;
    }
}

export async function hasHostPermissionForActiveTab() {
    const [activeTab] = await browser.tabs.query({
        active: true,
        currentWindow: true,
    });

    return await hasHostPermissionForSpecifiedTab(activeTab?.url);
}

export function setPageLanguage() {
    document.documentElement.lang = browser.i18n
        .getMessage("@@ui_locale")
        .replace("_", "-");
}
