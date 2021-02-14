module.exports = {
    getI18nString,
    setSyncStorage,
    getSyncStorage,
    reloadTab,
    dictEquality,
    tabQuery,
    tabSendmessage,
    addStorageChangeListener,
    openSettingsPage,
    setMessageListener,
    getExtensionUrl,
    camelize,
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

function addStorageChangeListener(listener) {
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
        console.error("Couldn't add setting change listener");
    }
}

function reloadTab(tab, callback) {
    if (chrome && chrome.tabs && chrome.tabs.reload) {
        chrome.tabs.reload(tab, callback);
    } else if (browser && browser.tabs && browser.tabs.reload) {
        browser.tabs.reload(tab, callback);
    } else {
        console.error("Couldn't reload tab.");
    }
}

function dictEquality(dict1, dict2) {
    let isEqual = true;

    for (var k in dict1) {
        if (Object.prototype.hasOwnProperty.call(dict1, k)) {
            isEqual = isEqual && dict1[k] === dict2[k];
        }
    }

    for (let k in dict2) {
        if (Object.prototype.hasOwnProperty.call(dict2, k)) {
            isEqual = isEqual && dict1[k] === dict2[k];
        }
    }

    return isEqual;
}

/**
 * Promise-based version of `chrome.tabs.query`.
 *
 * Mozilla does this by default in `browser.tabs` but Chrome is behind on this
 * sort of thing. Chrome won't even let us check if we're running in
 */
function tabQuery() {
    let myArgs = arguments;

    if (window.browser && browser.tabs && browser.tabs.query) {
        return browser.tabs.query.apply(this, arguments);
    }

    return new Promise(function (resolve) {
        let newArguments = Array.prototype.slice.call(myArgs);
        newArguments.push(resolve);
        chrome.tabs.query.apply(this, newArguments);
    });
}

/**
 * Promise-based version of `chrome.tabs.sendMessage`.
 */
function tabSendmessage() {
    let myArgs = arguments;

    if (window.browser && browser.tabs && browser.tabs.sendMessage) {
        return browser.tabs.sendMessage.apply(this, arguments);
    }

    return new Promise(function (resolve, reject) {
        let newArguments = Array.prototype.slice.call(myArgs);
        newArguments.push(function (response) {
            if (chrome.runtime.lastError !== undefined) {
                reject(chrome.runtime.lastError.message);
            }

            resolve(response);
        });
        chrome.tabs.sendMessage.apply(this, newArguments);
    });
}

function openSettingsPage() {
    if (chrome && chrome.tabs && chrome.tabs.create) {
        chrome.tabs.create({ url: "/options.html" });
        /* Open the settings page manually */
    } else if (browser && browser.runtime && browser.runtime.openOptionsPage) {
        browser.runtime.openOptionsPage();
        /* Have the browser open the settings page for us */
    } else {
        console.error("Can't open settings page");
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
    if (chrome && chrome.runtime && chrome.runtime.getURL) {
        return chrome.runtime.getURL(path);
    } else if (browser && browser.runtime && browser.runtime.getURL) {
        return browser.runtime.getURL(path);
    } else {
        console.error("Couldn't get extension URL");
    }
}

function camelize(str) {
    return str.toLowerCase().replace(/[^a-zA-Z0-9]+(.)/g, (m, chr) => {
        return chr.toUpperCase();
    });
}
