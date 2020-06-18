const {
    get_sync_storage,
    get_i18n_string,
    set_sync_storage,
} = require("./util.js");

get_sync_storage(["ruffle_enable", "ignore_optout"], function (data) {
    var play_flash_message = get_i18n_string("settings_ruffle_enable");
    var ignore_optout_message = get_i18n_string("settings_page_ignore_optout");
    var title_text = get_i18n_string("settings_page");
    var save_text = get_i18n_string("save_settings");
    var play_flash_label = document.getElementById("enablelabel");
    var ignore_optout_label = document.getElementById("ignorelabel");
    var play_flash_checkbox = document.getElementById("enable");
    var ignore_optout_checkbox = document.getElementById("ignoreoptout");
    var save_button = document.getElementById("save");
    var title = document.getElementById("title");
    title.innerHTML = title_text;
    document.title = title_text;
    play_flash_label.innerHTML = play_flash_message + "<br />";
    ignore_optout_label.innerHTML = ignore_optout_message + "<br />";
    save_button.value = save_text;
    if (data.ruffle_enable == "on") {
        play_flash_checkbox.checked = true;
    }
    if (data.ignore_optout == "on") {
        ignore_optout_checkbox.checked = true;
    }
    save_button.onclick = function () {
        set_sync_storage({
            ruffle_enable: play_flash_checkbox.checked ? "on" : "",
        });
        set_sync_storage({
            ignore_optout: ignore_optout_checkbox.checked ? "on" : "",
        });
        alert("Settings Saved");
    };
});
