const DEFAULT_SETTINGS = {
    ruffleEnable: true,
    ignoreOptout: false,
};

export function getI18nMessage(name) {
    if (chrome && chrome.i18n && chrome.i18n.getMessage) {
        return chrome.i18n.getMessage(name);
    } else if (browser && browser.i18n && browser.i18n.getMessage) {
        return browser.i18n.getMessage(name);
    } else {
        throw new Error(`Failed to get i18n message: ${name}`);
    }
}

export async function getSyncStorage(keys) {
    let data;
    if (
        chrome &&
        chrome.storage &&
        chrome.storage.sync &&
        chrome.storage.sync.get
    ) {
        data = await chrome.storage.sync.get(keys);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.sync &&
        browser.storage.sync.get
    ) {
        data = await browser.storage.sync.get(keys);
    } else {
        throw new Error(`Failed to get storage: ${keys}`);
    }

    // Copy over default settings if they don't exist yet.
    return { ...DEFAULT_SETTINGS, ...data };
}

export async function setSyncStorage(items) {
    if (
        chrome &&
        chrome.storage &&
        chrome.storage.sync &&
        chrome.storage.sync.set
    ) {
        return chrome.storage.sync.set(items);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.sync &&
        browser.storage.sync.set
    ) {
        return browser.storage.sync.set(items);
    } else {
        throw new Error(`Failed to set storage: ${items}`);
    }
}

export function addStorageChangeListener(listener) {
    if (
        chrome &&
        chrome.storage &&
        chrome.storage.onChanged &&
        chrome.storage.onChanged.addListener
    ) {
        chrome.storage.onChanged.addListener(listener);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.onChanged &&
        browser.storage.onChanged.addListener
    ) {
        browser.storage.onChanged.addListener(listener);
    } else {
        throw new Error("Failed to add storage change listener.");
    }
}

export async function reloadTab(tabId) {
    if (chrome && chrome.tabs && chrome.tabs.reload) {
        return chrome.tabs.reload(tabId);
    } else if (browser && browser.tabs && browser.tabs.reload) {
        return browser.tabs.reload(tabId);
    } else {
        throw new Error("Failed to reload tab.");
    }
}

export async function queryTabs(query) {
    if (chrome && chrome.tabs && chrome.tabs.query) {
        return browser.tabs.query(query);
    } else if (browser && browser.tabs && browser.tabs.query) {
        return browser.tabs.query(query);
    } else {
        throw new Error("Failed to query tabs.");
    }
}

export function sendMessageToTab(tabId, message, options) {
    if (chrome && chrome.tabs && chrome.tabs.sendMessage) {
        chrome.tabs.sendMessage(tabId, message, options);
    } else if (browser && browser.tabs && browser.tabs.sendMessage) {
        browser.tabs.sendMessage(tabId, message, options);
    } else {
        throw new Error("Failed to send message to tab.");
    }
}

export function openOptionsPage() {
    if (chrome && chrome.tabs && chrome.tabs.create) {
        chrome.tabs.create({ url: "/options.html" });
    } else if (browser && browser.runtime && browser.runtime.openOptionsPage) {
        browser.runtime.openOptionsPage();
    } else {
        throw new Error("Failed to open options page.");
    }
}

export function addMessageListener(listener) {
    if (
        chrome &&
        chrome.runtime &&
        chrome.runtime.onMessage &&
        chrome.runtime.onMessage.addListener
    ) {
        chrome.runtime.onMessage.addListener(listener);
    } else if (
        browser &&
        browser.runtime &&
        browser.runtime.onMessage &&
        browser.runtime.onMessage.addListener
    ) {
        browser.runtime.onMessage.addListener(listener);
    } else {
        throw new Error("Failed to add message listener.");
    }
}

export function getExtensionUrl(path) {
    if (chrome && chrome.runtime && chrome.runtime.getURL) {
        return chrome.runtime.getURL(path);
    } else if (browser && browser.runtime && browser.runtime.getURL) {
        return browser.runtime.getURL(path);
    } else {
        throw new Error("Faile to get extension URL.");
    }
}
