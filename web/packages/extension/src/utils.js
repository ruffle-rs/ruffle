const DEFAULT_OPTIONS = {
    ruffleEnable: true,
    ignoreOptout: false,
};

function promisify(func) {
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

export let i18n;
export let storage;
export let tabs;
export let runtime;
export let openOptionsPage;

if (typeof chrome !== "undefined") {
    i18n = {
        getMessage: (name) => chrome.i18n.getMessage(name),
    };

    const storageLocal = chrome.storage.local;
    const storageSync = chrome.storage.sync;
    storage = {
        local: {
            get: (keys) => promisify(storageLocal.get.bind(storageLocal, keys)),
            remove: (keys) =>
                promisify(storageLocal.remove.bind(storageLocal, keys)),
            set: (items) =>
                promisify(storageLocal.set.bind(storageLocal, items)),
        },
        sync: {
            get: (keys) => promisify(storageSync.get.bind(storageSync, keys)),
            remove: (keys) =>
                promisify(storageSync.remove.bind(storageSync, keys)),
            set: (items) => promisify(storageSync.set.bind(storageSync, items)),
        },
        onChanged: {
            addListener: (listener) =>
                chrome.storage.onChanged.addListener(listener),
        },
    };

    tabs = {
        reload: (tabId) =>
            promisify(chrome.tabs.reload.bind(chrome.tabs, tabId)),
        query: (query) => promisify(chrome.tabs.query.bind(chrome.tabs, query)),
        sendMessage: (tabId, message, options) =>
            promisify(
                chrome.tabs.sendMessage.bind(
                    chrome.tabs,
                    tabId,
                    message,
                    options
                )
            ),
    };

    runtime = {
        onMessage: {
            addListener: (listener) =>
                chrome.runtime.onMessage.addListener(listener),
        },
        getURL: (path) => chrome.runtime.getURL(path),
    };

    openOptionsPage = () => chrome.tabs.create({ url: "/options.html" });
} else if (typeof browser !== "undefined") {
    i18n = {
        getMessage: (name) => browser.i18n.getMessage(name),
    };

    storage = {
        local: {
            get: (keys) => browser.storage.local.get(keys),
            remove: (keys) => browser.storage.local.set(keys),
            set: (items) => browser.storage.local.set(items),
        },
        sync: {
            get: (keys) => browser.storage.sync.get(keys),
            remove: (keys) => browser.storage.sync.set(keys),
            set: (items) => browser.storage.sync.set(items),
        },
        onChanged: {
            addListener: (listener) =>
                browser.storage.onChanged.addListener(listener),
        },
    };

    tabs = {
        reload: (tabId) => browser.tabs.reload(tabId),
        query: (query) => browser.tabs.query(query),
        sendMessage: (tabId, message, options) =>
            browser.tabs.sendMessage(tabId, message, options),
    };

    runtime = {
        onMessage: {
            addListener: (listener) =>
                browser.runtime.onMessage.addListener(listener),
        },
        getURL: (path) => browser.runtime.getURL(path),
    };

    openOptionsPage = () => browser.runtime.openOptionsPage();
} else {
    throw new Error("Extension API not found.");
}

export async function getOptions(keys) {
    const options = await storage.sync.get(keys);

    // Copy over default options if they don't exist yet.
    return { ...DEFAULT_OPTIONS, ...options };
}
