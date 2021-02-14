const { getSyncStorage, getI18nString, setSyncStorage } = require("./util.js");

getSyncStorage(["ruffleEnable", "ignoreOptout"], function (data) {
    var playFlashMessage = getI18nString("settings_ruffle_enable");
    var ignoreOptoutMessage = getI18nString("settings_page_ignore_optout");
    var titleText = getI18nString("settings_page");
    var saveText = getI18nString("save_settings");
    var playFlashLabel = document.getElementById("enablelabel");
    var ignoreOptoutLabel = document.getElementById("ignorelabel");
    var playFlashCheckbox = document.getElementById("enable");
    var ignoreOptoutCheckbox = document.getElementById("ignoreoptout");
    var saveButton = document.getElementById("save");
    var title = document.getElementById("title");
    title.innerHTML = titleText;
    document.title = titleText;
    playFlashLabel.innerHTML = playFlashMessage + "<br />";
    ignoreOptoutLabel.innerHTML = ignoreOptoutMessage + "<br />";
    saveButton.value = saveText;
    playFlashCheckbox.checked = data.ruffleEnable;
    ignoreOptoutCheckbox.checked = data.ignoreOptout;
    saveButton.onclick = function () {
        setSyncStorage({
            ruffleEnable: playFlashCheckbox.checked,
            ignoreOptout: ignoreOptoutCheckbox.checked,
        });
        alert(getI18nString("settings_saved"));
    };
});
