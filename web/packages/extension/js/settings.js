function get_sync_storage(key, callback) {
    if (
        chrome &&
        chrome.storage &&
        chrome.storage.sync &&
        chrome.storage.sync.get
    ) {
        chrome.storage.sync.get(key, callback);
    } else if (
        browser &&
        browser.storage &&
        browser.storage.sync &&
        browser.storage.sync.get
    ) {
        browser.storage.sync.get(key, callback);
    } else {
        console.error("Couldn't read setting: " + key);
    }
}

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

get_sync_storage(["ruffle_enable", "ignore_optout"], function (data) {
    var play_flash_message = get_i18n_string("settings_ruffle_enable");
    var ignore_optout_message = get_i18n_string("settings_page_ignore_optout");
    var play_flash_label = document.getElementById("enablelabel");
    var ignore_optout_label = document.getElementById("ignorelabel");
    var play_flash_checkbox = document.getElementById("enable");
    var ignore_optout_checkbox = document.getElementById("ignoreoptout");
    var save_button = document.getElementById("save");
    console.log(play_flash_message);
    console.log(ignore_optout_message);
    play_flash_label.innerHTML = play_flash_message + "<br />";
    ignore_optout_label.innerHTML = ignore_optout_message + "<br />";
    if (data.ruffle_enable == "on") {
        play_flash_checkbox.checked = true;
    }
    if (data.ignore_optout == "on") {
        ignore_optout_checkbox.checked = true;
    }
    save_button.onclick = function () {
        set_sync_storage(
            { ruffle_enable: play_flash_checkbox.checked ? "on" : "" },
            function () {
                console.log("ruffle_enable updated");
            }
        );
        set_sync_storage(
            { ignore_optout: ignore_optout_checkbox.checked ? "on" : "" },
            function () {
                console.log("ignore_optout updated");
            }
        );
        alert("Settings Saved");
    };
});
