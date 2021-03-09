module.exports = {
    getI18nString,
    setSyncStorage,
    getSyncStorage,
    setMessageListener,
    getExtensionUrl,
};

// List of defaults for all settings.
const DEFAULT_SETTINGS = {
    ruffleEnable: true,
    ignoreOptout: false,
};

function getI18nString(key) {
    if (chrome && chrome.i18n && chrome.i18n.getMessage) {
        return chrome.i18n.getMessage(key);
    } else if (browser && browser.i18n && browser.i18n.getMessage) {
        return browser.i18n.getMessage(key);
    } else {
        console.error("Can't get i18n message: " + key);
    }
}

function setSyncStorage(key) {
    if (
        chrome &&
        chrome.storage &&
        chrome.storage.sync &&
        chrome.storage.sync.set
    ) {
        chrome.storage.sync.set(key);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.sync &&
        browser.storage.sync.set
    ) {
        browser.storage.sync.set(key);
    } else {
        console.error("Can't set settings.");
    }
}

function getSyncStorage(key, callback) {
    // Create array of keys so that we can grab the defaults, if necessary.
    let dataType = typeof key;
    let keys;
    if (dataType == "string") {
        keys = [key];
    } else if (Array.isArray(key)) {
        keys = key;
    } else {
        keys = Object.keys(key);
    }

    // Copy over default settings if they don't exist yet.
    let callbackWithDefault = (data) => {
        for (const k of keys) {
            if (data[k] === undefined) {
                data[k] = DEFAULT_SETTINGS[k];
            }
        }
        return callback(data);
    };

    if (
        chrome &&
        chrome.storage &&
        chrome.storage.sync &&
        chrome.storage.sync.get
    ) {
        chrome.storage.sync.get(key, callbackWithDefault);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.sync &&
        browser.storage.sync.get
    ) {
        browser.storage.sync.get(key, callbackWithDefault);
    } else {
        console.error("Couldn't read setting: " + key);
    }
}

function setMessageListener(listener) {
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
        console.error("Couldn't add message listener");
    }
}

function getExtensionUrl(path) {
    if (chrome && chrome.extension && chrome.extension.getURL) {
        return chrome.extension.getURL(path);
    } else if (browser && browser.runtime && browser.runtime.getURL) {
        return browser.runtime.getURL(path);
    } else {
        console.error("Couldn't get extension URL");
    }
}
