const DEFAULT_OPTIONS = {
    ruffleEnable: true,
    ignoreOptout: false,
};

export let i18n: any;
export let storage: any;
export let tabs: {
    reload(tabId: number): Promise<void>,
    query(query: chrome.tabs.QueryInfo & browser.tabs._QueryQueryInfo): Promise<chrome.tabs.Tab[] | browser.tabs.Tab[]>,
    sendMessage(tabId: number, message: any, options?: chrome.tabs.MessageSendOptions & browser.tabs._SendMessageOptions): Promise<any>,
};
export let runtime: any;
export let openOptionsPage: any;

function promisify<T>(func: (callback: (result?: T) => void) => void): Promise<T> {
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

if (typeof chrome !== "undefined") {
    i18n = {
        getMessage: (name: string) => chrome.i18n.getMessage(name),
    };

    storage = {
        local: {
            get: (keys: string[]) =>
                promisify((cb) => chrome.storage.local.get(keys, cb)),
            remove: (keys: string[]) =>
                promisify((cb) => chrome.storage.local.remove(keys, cb)),
            set: (items: object) =>
                promisify((cb) => chrome.storage.local.set(items, cb)),
        },
        sync: {
            get: (keys: string[]) => promisify((cb) => chrome.storage.sync.get(keys, cb)),
            remove: (keys: string[]) =>
                promisify((cb) => chrome.storage.sync.remove(keys, cb)),
            set: (items: object) =>
                promisify((cb) => chrome.storage.sync.set(items, cb)),
        },
        onChanged: {
            addListener: (listener: (changes: chrome.storage.StorageChange, areaName: string) => void) =>
                chrome.storage.onChanged.addListener(listener),
        },
    };

    tabs = {
        reload: (tabId: number) => promisify((cb) => chrome.tabs.reload(tabId, undefined, cb)),
        query: (query: chrome.tabs.QueryInfo) => promisify((cb) => chrome.tabs.query(query, cb)),
        sendMessage: (tabId: number, message: any, options: chrome.tabs.MessageSendOptions) =>
            promisify((cb) => chrome.tabs.sendMessage(tabId, message, options, cb)),
    };

    runtime = {
        onMessage: {
            addListener: (listener: (message: any, sender: chrome.runtime.MessageSender, sendResponse: (response?: any) => void) => void) =>
                chrome.runtime.onMessage.addListener(listener),
        },
        getURL: (path: string) => chrome.runtime.getURL(path),
    };

    openOptionsPage = () => chrome.tabs.create({ url: "/options.html" });
} else if (typeof browser !== "undefined") {
    i18n = {
        getMessage: (name: string) => browser.i18n.getMessage(name),
    };

    storage = {
        local: {
            get: (keys: string[]) => browser.storage.local.get(keys),
            remove: (keys: string[]) => browser.storage.local.remove(keys),
            set: (items: Record<string, any>) => browser.storage.local.set(items),
        },
        sync: {
            get: (keys: string[]) => browser.storage.sync.get(keys),
            remove: (keys: string[]) => browser.storage.sync.remove(keys),
            set: (items: Record<string, any>) => browser.storage.sync.set(items),
        },
        onChanged: {
            addListener: (listener: (changes: Record<string, browser.storage.StorageChange>, areaName: string) => void) =>
                browser.storage.onChanged.addListener(listener),
        },
    };

    tabs = {
        reload: (tabId: number) => browser.tabs.reload(tabId),
        query: (query: browser.tabs._QueryQueryInfo) => browser.tabs.query(query),
        sendMessage: (tabId: number, message: any, options: browser.tabs._SendMessageOptions) =>
            browser.tabs.sendMessage(tabId, message, options),
    };

    runtime = {
        onMessage: {
            addListener: (listener: (message: any, sender: browser.runtime.MessageSender, sendResponse: (response?: any) => void) => boolean | Promise<any> | void) =>
                browser.runtime.onMessage.addListener(listener),
        },
        getURL: (path: string) => browser.runtime.getURL(path),
    };

    openOptionsPage = () => browser.runtime.openOptionsPage();
} else {
    throw new Error("Extension API not found.");
}

export async function getOptions(keys: string[]) {
    const options = await storage.sync.get(keys);

    // Copy over default options if they don't exist yet.
    return { ...DEFAULT_OPTIONS, ...options };
}
