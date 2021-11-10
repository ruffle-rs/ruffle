import { Options } from "./common";
import { LogLevel } from "ruffle-core";

const DEFAULT_OPTIONS: Options = {
    ruffleEnable: true,
    ignoreOptout: false,
    warnOnUnsupportedContent: true,
    logLevel: LogLevel.Error,
    showSwfDownload: false,
};

export let i18n: {
    getMessage: (name: string) => string;
};

interface StorageArea {
    get: (keys?: string[]) => Promise<Record<string, unknown>>;
    remove: (keys: string[]) => Promise<void>;
    set: (items: Record<string, unknown>) => Promise<void>;
}

export let storage: {
    local: StorageArea;
    sync: StorageArea;
    onChanged: {
        addListener: (
            listener: (
                changes:
                    | Record<string, chrome.storage.StorageChange>
                    | Record<string, browser.storage.StorageChange>,
                areaName: string
            ) => void
        ) => void;
    };
};

export let tabs: {
    reload: (tabId: number) => Promise<void>;
    query: (
        query: chrome.tabs.QueryInfo & browser.tabs._QueryQueryInfo
    ) => Promise<chrome.tabs.Tab[] | browser.tabs.Tab[]>;
    sendMessage: (
        tabId: number,
        message: unknown,
        options?: chrome.tabs.MessageSendOptions &
            browser.tabs._SendMessageOptions
    ) => Promise<any>; // eslint-disable-line @typescript-eslint/no-explicit-any
};

export let runtime: {
    onMessage: {
        addListener: (
            listener: (
                message: unknown,
                sender:
                    | chrome.runtime.MessageSender
                    | browser.runtime.MessageSender,
                sendResponse: (response?: unknown) => void
            ) => void
        ) => void;
    };
    getURL: (path: string) => string;
};

export let openOptionsPage: () => Promise<void>;

function promisify<T>(
    func: (callback: (result?: T) => void) => void
): Promise<T> {
    return new Promise((resolve, reject) => {
        func((result) => {
            const error = chrome.runtime.lastError;
            if (error) {
                reject(error);
            } else {
                resolve(result!);
            }
        });
    });
}

function promisifyStorageArea(
    storage: chrome.storage.StorageArea
): StorageArea {
    return {
        get: (keys?: string[]) =>
            promisify((cb) => storage.get(keys || null, cb)),
        remove: (keys: string[]) => promisify((cb) => storage.remove(keys, cb)),
        set: (items: Record<string, unknown>) =>
            promisify((cb) => storage.set(items, cb)),
    };
}

if (typeof chrome !== "undefined") {
    i18n = chrome.i18n;

    storage = {
        local: promisifyStorageArea(chrome.storage.local),
        sync: promisifyStorageArea(chrome.storage.sync),
        onChanged: {
            addListener: (
                listener: (
                    changes: Record<string, chrome.storage.StorageChange>,
                    areaName: string
                ) => void
            ) => chrome.storage.onChanged.addListener(listener),
        },
    };

    tabs = {
        reload: (tabId: number) =>
            promisify((cb) => chrome.tabs.reload(tabId, undefined, cb)),
        query: (query: chrome.tabs.QueryInfo) =>
            promisify((cb) => chrome.tabs.query(query, cb)),
        sendMessage: (
            tabId: number,
            message: unknown,
            options?: chrome.tabs.MessageSendOptions
        ) =>
            promisify((cb) =>
                chrome.tabs.sendMessage(tabId, message, options || {}, cb)
            ),
    };

    runtime = chrome.runtime;

    openOptionsPage = () =>
        promisify((cb: () => void) =>
            chrome.tabs.create({ url: "/options.html" }, cb)
        );
} else if (typeof browser !== "undefined") {
    i18n = browser.i18n;
    storage = browser.storage;
    tabs = browser.tabs;
    runtime = browser.runtime;
    openOptionsPage = () => browser.runtime.openOptionsPage();
} else {
    throw new Error("Extension API not found.");
}

export async function getOptions(): Promise<Options> {
    const options = await storage.sync.get();

    // Copy over default options if they don't exist yet.
    return { ...DEFAULT_OPTIONS, ...options };
}
