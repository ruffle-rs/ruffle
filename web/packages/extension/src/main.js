const {
    getI18nString,
    setSyncStorage,
    getSyncStorage,
    addStorageChangeListener,
    reloadTab,
    dictEquality,
    tabQuery,
    tabSendmessage,
    openSettingsPage,
    camelize,
} = require("./util.js");

let settingsDict = {},
    tabSettings = {},
    reloadButton,
    activeTab;

function onSettingsChangeIntent() {
    let isDifferent = !dictEquality(settingsDict, tabSettings);

    if (reloadButton !== undefined) {
        reloadButton.disabled = !isDifferent;
    }
}

function bindBooleanSetting(checkboxElem) {
    let name = checkboxElem.name,
        label = checkboxElem.nextSibling;

    label.textContent = getI18nString("settings_" + name);
    name = camelize(name);

    getSyncStorage(name, function (items) {
        checkboxElem.checked = items[name];
        settingsDict[name] = items[name];
        onSettingsChangeIntent();
    });

    addStorageChangeListener(function (changes) {
        if (Object.prototype.hasOwnProperty.call(changes, name)) {
            checkboxElem.checked = changes[name].newValue;
            settingsDict[name] = changes[name].newValue;
            onSettingsChangeIntent();
        }
    });

    checkboxElem.addEventListener("click", function () {
        let setting = {};
        setting[name] = checkboxElem.checked;
        settingsDict[name] = setting[name];
        onSettingsChangeIntent();

        setSyncStorage(setting);
    });
}

function bindSettingsApplyButton(elem) {
    elem.textContent = getI18nString("action_" + elem.id);
    elem.disabled = true;

    elem.addEventListener("click", function () {
        reloadTab(activeTab.id, function () {
            window.setInterval(queryCurrentTab, 1000);
        });
    });

    reloadButton = elem;
}

let gotStatus = false;

async function queryCurrentTab() {
    let ruffleStatus = document.getElementById("ruffle_status");
    if (ruffleStatus === null) {
        /*debugger;*/
    }

    if (!gotStatus) {
        ruffleStatus.textContent = getI18nString("status_init");
    }

    let tabs = null;

    try {
        tabs = await tabQuery({
            currentWindow: true,
            active: true,
        });

        if (tabs.length < 1) {
            ruffleStatus.textContent = getI18nString("status_no_tabs");
            return;
        }

        if (tabs.length > 1) {
            console.warn(
                "Got " + tabs.length + " tabs in response to active tab query"
            );
        }
    } catch (e) {
        ruffleStatus.textContent = getI18nString("status_tabs_error");
        throw e;
    }

    try {
        activeTab = tabs[0];

        ruffleStatus.textContent = getI18nString("status_message_init");

        let resp = await tabSendmessage(activeTab.id, {
            action: "get_page_options",
        });

        tabSettings = resp.tabSettings;
        onSettingsChangeIntent();

        if (resp !== undefined && resp.loaded) {
            ruffleStatus.textContent = getI18nString("status_result_running");
        } else if (resp !== undefined && !resp.loaded) {
            if (tabSettings.ruffleEnable) {
                ruffleStatus.textContent = getI18nString(
                    "status_result_optout"
                );
            } else {
                ruffleStatus.textContent = getI18nString(
                    "status_result_disabled"
                );
            }
        } else {
            ruffleStatus.textContent = getI18nString("status_result_error");
        }
    } catch (e) {
        ruffleStatus.textContent = getI18nString("status_result_protected");
        if (reloadButton) {
            reloadButton.disabled = true;
        }
        throw e;
    }
}

document.addEventListener("DOMContentLoaded", function () {
    var settingsButton = document.getElementById("settingsbutton");
    bindBooleanSetting(document.getElementById("ruffle_enable"));
    bindBooleanSetting(document.getElementById("ignore_optout"));
    bindSettingsApplyButton(document.getElementById("reload"));
    settingsButton.innerHTML = getI18nString("open_settings_page");
    settingsButton.onclick = openSettingsPage;

    queryCurrentTab();
});
