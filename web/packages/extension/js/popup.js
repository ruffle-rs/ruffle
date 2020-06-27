const {
    get_i18n_string,
    set_sync_storage,
    get_sync_storage,
    add_storage_change_listener,
    reload_tab,
    dict_equality,
    tab_query,
    tab_sendmessage,
    open_settings_page,
} = require("./util.js");

let settings_dict = {},
    tab_settings = {},
    reload_button,
    active_tab;

function on_settings_change_intent() {
    let is_different = !dict_equality(settings_dict, tab_settings);

    if (reload_button !== undefined) {
        reload_button.disabled = !is_different;
    }
}

function bind_boolean_setting(checkbox_elem) {
    let name = checkbox_elem.name,
        default_val = checkbox_elem.checked,
        get_obj = {},
        label = checkbox_elem.nextSibling;

    label.textContent = get_i18n_string("settings_" + name);

    get_obj[name] = default_val;

    get_sync_storage(get_obj, function (items) {
        checkbox_elem.checked = items[name] === checkbox_elem.value;
        settings_dict[name] = items[name];
        on_settings_change_intent();
    });

    add_storage_change_listener(function (changes) {
        if (Object.prototype.hasOwnProperty.call(changes, name)) {
            checkbox_elem.checked =
                changes[name].newValue === checkbox_elem.value;
            settings_dict[name] = changes[name].newValue;
            on_settings_change_intent();
        }
    });

    checkbox_elem.addEventListener("click", function () {
        let setting = {};
        setting[name] = checkbox_elem.checked ? "on" : "";
        settings_dict[name] = setting[name];
        on_settings_change_intent();

        set_sync_storage(setting);
    });
}

function bind_settings_apply_button(elem) {
    elem.textContent = get_i18n_string("action_" + elem.id);
    elem.disabled = true;

    elem.addEventListener("click", function () {
        reload_tab(active_tab.id, function () {
            window.setInterval(query_current_tab, 1000);
        });
    });

    reload_button = elem;
}

let got_status = false;

async function query_current_tab() {
    let ruffle_status = document.getElementById("ruffle_status");
    if (ruffle_status === null) {
        /*debugger;*/
    }

    if (!got_status) {
        ruffle_status.textContent = get_i18n_string("status_init");
    }

    let tabs = null;

    try {
        tabs = await tab_query({
            currentWindow: true,
            active: true,
        });

        if (tabs.length < 1) {
            ruffle_status.textContent = get_i18n_string("status_no_tabs");
            return;
        }

        if (tabs.length > 1) {
            console.warn(
                "Got " + tabs.length + " tabs in response to active tab query"
            );
        }
    } catch (e) {
        ruffle_status.textContent = get_i18n_string("status_tabs_error");
        throw e;
    }

    try {
        active_tab = tabs[0];

        ruffle_status.textContent = get_i18n_string("status_message_init");

        let resp = await tab_sendmessage(active_tab.id, {
            action: "get_page_options",
        });

        tab_settings = resp.tab_settings;
        on_settings_change_intent();

        if (resp !== undefined && resp.loaded) {
            ruffle_status.textContent = get_i18n_string(
                "status_result_running"
            );
        } else if (resp !== undefined && !resp.loaded) {
            if (tab_settings.ruffle_enable === "on") {
                ruffle_status.textContent = get_i18n_string(
                    "status_result_optout"
                );
            } else {
                ruffle_status.textContent = get_i18n_string(
                    "status_result_disabled"
                );
            }
        } else {
            ruffle_status.textContent = get_i18n_string("status_result_error");
        }
    } catch (e) {
        ruffle_status.textContent = get_i18n_string("status_result_protected");
        if (reload_button) {
            reload_button.disabled = true;
        }
        throw e;
    }
}

document.addEventListener("DOMContentLoaded", function () {
    var settings_button = document.getElementById("settingsbutton");
    bind_boolean_setting(document.getElementById("ruffle_enable"));
    bind_boolean_setting(document.getElementById("ignore_optout"));
    bind_settings_apply_button(document.getElementById("reload"));
    settings_button.innerHTML = get_i18n_string("open_settings_page");
    settings_button.onclick = open_settings_page;

    query_current_tab();
});
