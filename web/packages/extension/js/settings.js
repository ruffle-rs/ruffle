chrome.storage.sync.get(["ruffle_enable", "ignore_optout"], function (data) {
    var play_flash_message = chrome.i18n.getMessage("settings_ruffle_enable");
    var ignore_optout_message = chrome.i18n.getMessage(
        "settings_page_ignore_optout"
    );
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
        chrome.storage.sync.set(
            { ruffle_enable: play_flash_checkbox.checked ? "on" : "" },
            function () {
                console.log("ruffle_enable updated");
            }
        );
        chrome.storage.sync.set(
            { ignore_optout: ignore_optout_checkbox.checked ? "on" : "" },
            function () {
                console.log("ignore_optout updated");
            }
        );
        alert("Settings Saved");
    };
});
