module.exports = {
    get_i18n_string,
    set_sync_storage,
    get_sync_storage,
    reload_tab,
    dict_equality,
    tab_query,
    tab_sendmessage,
    add_storage_change_listener,
    open_settings_page,
    set_message_listener,
    get_extension_url,
};

// List of defaults for all settings.
const DEFAULT_SETTINGS = {
    ruffle_enable: true,
    ignore_output: false,
};

function get_i18n_string(key) {
    if (chrome && chrome.i18n && chrome.i18n.getMessage) {
        return chrome.i18n.getMessage(key);
    } else if (browser && browser.i18n && browser.i18n.getMessage) {
        return browser.i18n.getMessage(key);
    } else {
        console.error("Can't get i18n message: " + key);
    }
}

function set_sync_storage(key) {
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

function get_sync_storage(key, callback) {
    // Create array of keys so that we can grab the defaults, if necessary.
    let data_type = typeof key;
    let keys;
    if (data_type == "string") {
        keys = [key];
    } else if (Array.isArray(key)) {
        keys = key;
    } else {
        keys = Object.keys(key);
    }

    // Copy over default settings if they don't exist yet.
    let callback_with_default = (data) => {
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
        chrome.storage.sync.get(key, callback_with_default);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.sync &&
        browser.storage.sync.get
    ) {
        browser.storage.sync.get(key, callback_with_default);
    } else {
        console.error("Couldn't read setting: " + key);
    }
}

function add_storage_change_listener(listener) {
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

function reload_tab(tab, callback) {
    if (chrome && chrome.tabs && chrome.tabs.reload) {
        chrome.tabs.reload(tab, callback);
    } else if (browser && browser.tabs && browser.tabs.reload) {
        browser.tabs.reload(tab, callback);
    } else {
        console.error("Couldn't reload tab.");
    }
}

function dict_equality(dict1, dict2) {
    let is_equal = true;

    for (var k in dict1) {
        if (Object.prototype.hasOwnProperty.call(dict1, k)) {
            is_equal = is_equal && dict1[k] === dict2[k];
        }
    }

    for (let k in dict2) {
        if (Object.prototype.hasOwnProperty.call(dict2, k)) {
            is_equal = is_equal && dict1[k] === dict2[k];
        }
    }

    return is_equal;
}

/**
 * Promise-based version of `chrome.tabs.query`.
 *
 * Mozilla does this by default in `browser.tabs` but Chrome is behind on this
 * sort of thing. Chrome won't even let us check if we're running in
 */
function tab_query() {
    let my_args = arguments;

    if (window.browser && browser.tabs && browser.tabs.query) {
        return browser.tabs.query.apply(this, arguments);
    }

    return new Promise(function (resolve) {
        let new_arguments = Array.prototype.slice.call(my_args);
        new_arguments.push(resolve);
        chrome.tabs.query.apply(this, new_arguments);
    });
}

/**
 * Promise-based version of `chrome.tabs.sendMessage`.
 */
function tab_sendmessage() {
    let my_args = arguments;

    if (window.browser && browser.tabs && browser.tabs.sendMessage) {
        return browser.tabs.sendMessage.apply(this, arguments);
    }

    return new Promise(function (resolve, reject) {
        let new_arguments = Array.prototype.slice.call(my_args);
        new_arguments.push(function (response) {
            if (chrome.runtime.lastError !== undefined) {
                reject(chrome.runtime.lastError.message);
            }

            resolve(response);
        });
        chrome.tabs.sendMessage.apply(this, new_arguments);
    });
}

function open_settings_page() {
    if (chrome && chrome.tabs && chrome.tabs.create) {
        chrome.tabs.create({ url: "/settings.htm" });
        /* Open the settings page manually */
    } else if (browser && browser.runtime && browser.runtime.openOptionsPage) {
        browser.runtime.openOptionsPage();
        /* Have the browser open the settings page for us */
    } else {
        console.error("Can't open settings page");
    }
}

function set_message_listener(listener) {
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

function get_extension_url() {
    if (chrome && chrome.extension && chrome.extension.getURL) {
        return chrome.extension
            .getURL("dist/ruffle.js")
            .replace("dist/ruffle.js", "");
    } else if (browser && browser.runtime && browser.runtime.getURL) {
        return browser.runtime
            .getURL("dist/ruffle.js")
            .replace("dist/ruffle.js", "");
    } else {
        console.error("Couldn't get extension URL");
    }
}
