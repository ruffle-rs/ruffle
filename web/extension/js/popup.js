function bind_boolean_setting(checkbox_elem) {
    let name = checkbox_elem.name,
        default_val = checkbox_elem.checked,
        get_obj = {};
    
    get_obj[name] = default_val;

    chrome.storage.sync.get(get_obj, function (items) {
        checkbox_elem.checked = items[name] === checkbox_elem.value;
    });

    chrome.storage.onChanged.addListener(function (changes, namespace) {
        if (changes.hasOwnProperty(name)) {
            checkbox_elem.checked = changes[name].newValue === checkbox_elem.value;
        }
    });

    checkbox_elem.addEventListener("click", function (e) {
        let setting = {};
        setting[name] = checkbox_elem.checked ? checkbox_elem.value : "";

        chrome.storage.sync.set(setting);
    });
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

    return new Promise(function (resolve, reject) {
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

document.addEventListener("DOMContentLoaded", async function (e) {
    bind_boolean_setting(document.getElementById("ruffle_enable"));
    bind_boolean_setting(document.getElementById("ignore_optout"));
    
    let ruffle_status = document.getElementById("ruffle_status");
    if (ruffle_status === null) {
        debugger;
    }
    
    ruffle_status.textContent = "Reading current tab...";
    let tabs = null;

    try {
        tabs = await tab_query({
            "currentWindow": true,
            "active": true
        });

        if (tabs.length < 1) {
            ruffle_status.textContent = "There is no active tab.";
            return;
        }

        if (tabs.length > 1) {
            console.warn("Got " + tabs.length + " tabs in response to active tab query");
        }
    } catch (e) {
        ruffle_status.textContent = "An error occured when looking up the current tab.";
        throw e;
    }

    try {
        let active_tab = tabs[0];

        ruffle_status.textContent = "Checking Ruffle status on current tab...";

        let resp = await tab_sendmessage(active_tab.id, {"action": "get_page_options"});
        console.log(resp);
        if (resp !== undefined && resp.loaded) {
            ruffle_status.textContent = "Ruffle is loaded and running Flash content on the current tab.";
        } else if (resp !== undefined && !resp.loaded) {
            ruffle_status.textContent = "Ruffle is not loaded because the current page has marked itself as incompatible.";
        } else {
            ruffle_status.textContent = "Current tab responded with invalid data.";
        }
    } catch (e) {
        ruffle_status.textContent = "Ruffle is not loaded on the current tab.";
        throw e;
    }
});