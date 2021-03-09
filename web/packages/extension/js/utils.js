const DEFAULT_OPTIONS = {
    ruffleEnable: true,
    ignoreOptout: false,
};

export let i18n;
export let storage;
export let tabs;
export let runtime;
export let openOptionsPage;

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

if (typeof chrome !== "undefined") {
    i18n = {
        getMessage: (name) => chrome.i18n.getMessage(name),
    };

    storage = {
        local: {
            get: (keys) =>
                promisify((cb) => chrome.storage.local.get(keys, cb)),
            remove: (keys) =>
                promisify((cb) => chrome.storage.local.remove(keys, cb)),
            set: (items) =>
                promisify((cb) => chrome.storage.local.set(items, cb)),
        },
        sync: {
            get: (keys) => promisify((cb) => chrome.storage.sync.get(keys, cb)),
            remove: (keys) =>
                promisify((cb) => chrome.storage.sync.remove(keys, cb)),
            set: (items) =>
                promisify((cb) => chrome.storage.sync.set(items, cb)),
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

    openOptionsPage = () => chrome.tabs.create({ url: "/settings.html" });
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
