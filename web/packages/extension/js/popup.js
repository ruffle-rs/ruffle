let settings_dict = {},
    tab_settings = {},
    reload_button,
    active_tab;

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

function on_settings_change_intent() {
    let is_different = !dict_equality(settings_dict, tab_settings);

    console.log(settings_dict);
    console.log(tab_settings);
    console.log(is_different);

    if (reload_button !== undefined) {
        reload_button.disabled = !is_different;
    }
}

function bind_boolean_setting(checkbox_elem) {
    let name = checkbox_elem.name,
        default_val = checkbox_elem.checked,
        get_obj = {},
        label = checkbox_elem.nextSibling;

    label.textContent = chrome.i18n.getMessage("settings_" + name);

    get_obj[name] = default_val;

    chrome.storage.sync.get(get_obj, function (items) {
        checkbox_elem.checked = items[name] === checkbox_elem.value;
        settings_dict[name] = items[name];
        on_settings_change_intent();
    });

    chrome.storage.onChanged.addListener(function (changes) {
        if (Object.prototype.hasOwnProperty.call(changes, name)) {
            checkbox_elem.checked =
                changes[name].newValue === checkbox_elem.value;
            settings_dict[name] = changes[name].newValue;
            on_settings_change_intent();
        }
    });

    checkbox_elem.addEventListener("click", function () {
        let setting = {};
        setting[name] = checkbox_elem.checked ? checkbox_elem.value : "";
        settings_dict[name] = setting[name];
        on_settings_change_intent();

        chrome.storage.sync.set(setting);
    });
}

function bind_settings_apply_button(elem) {
    elem.textContent = chrome.i18n.getMessage("action_" + elem.id);
    elem.disabled = true;

    elem.addEventListener("click", function () {
        chrome.tabs.reload(active_tab.id, function () {
            window.setTimeout(query_current_tab, 1000);
        });
    });

    reload_button = elem;
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

async function query_current_tab() {
    let ruffle_status = document.getElementById("ruffle_status");
    if (ruffle_status === null) {
        /*debugger;*/
    }

    ruffle_status.textContent = chrome.i18n.getMessage("status_init");
    let tabs = null;

    try {
        tabs = await tab_query({
            currentWindow: true,
            active: true,
        });

        if (tabs.length < 1) {
            ruffle_status.textContent = chrome.i18n.getMessage(
                "status_no_tabs"
            );
            return;
        }

        if (tabs.length > 1) {
            console.warn(
                "Got " + tabs.length + " tabs in response to active tab query"
            );
        }
    } catch (e) {
        ruffle_status.textContent = chrome.i18n.getMessage("status_tabs_error");
        throw e;
    }

    try {
        active_tab = tabs[0];

        ruffle_status.textContent = chrome.i18n.getMessage(
            "status_message_init"
        );

        let resp = await tab_sendmessage(active_tab.id, {
            action: "get_page_options",
        });
        console.log(resp);

        tab_settings = resp.tab_settings;
        on_settings_change_intent();

        if (resp !== undefined && resp.loaded) {
            ruffle_status.textContent = chrome.i18n.getMessage(
                "status_result_running"
            );
        } else if (resp !== undefined && !resp.loaded) {
            if (tab_settings.ruffle_enable === "on") {
                ruffle_status.textContent = chrome.i18n.getMessage(
                    "status_result_optout"
                );
            } else {
                ruffle_status.textContent = chrome.i18n.getMessage(
                    "status_result_disabled"
                );
            }
        } else {
            ruffle_status.textContent = chrome.i18n.getMessage(
                "status_result_error"
            );
        }
    } catch (e) {
        ruffle_status.textContent = chrome.i18n.getMessage(
            "status_result_protected"
        );
        throw e;
    }
}

document.addEventListener("DOMContentLoaded", function () {
    bind_boolean_setting(document.getElementById("ruffle_enable"));
    bind_boolean_setting(document.getElementById("ignore_optout"));
    bind_settings_apply_button(document.getElementById("reload"));

    query_current_tab();
});
