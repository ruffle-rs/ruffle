import {
    getI18nMessage,
    addStorageChangeListener,
    sendMessageToTab,
    openOptionsPage,
    reloadTab,
    queryTabs,
} from "./utils";
import { bindBooleanOptions } from "./common";

let activeTab;
let reloadButton;
let tabOptions;

async function queryStatus(listener) {
    listener("status_init");

    let tabs;
    try {
        tabs = await queryTabs({
            currentWindow: true,
            active: true,
        });

        if (tabs.length < 1) {
            listener("status_no_tabs");
            return;
        }

        if (tabs.length > 1) {
            throw new Error(
                `Got ${tabs.length} tabs in response to active tab query.`
            );
        }
    } catch (e) {
        listener("status_tabs_error");
        return;
    }

    activeTab = tabs[0];
    listener("status_message_init");

    let response;
    try {
        response = await sendMessageToTab(activeTab.id, {
            action: "get_page_options",
        });
    } catch (e) {
        listener("status_result_protected");
        reloadButton.disabled = true;
        return;
    }

    if (!response) {
        listener("status_result_error");
        return;
    }

    tabOptions = response.tabSettings;
    optionsChanged();

    if (response.loaded) {
        listener("status_result_running");
    } else if (response.tabSettings.ruffleEnable) {
        listener("status_result_optout");
    } else {
        listener("status_result_disabled");
    }
}

function objectsEqual(x, y) {
    for (const [key, value] of Object.entries(x)) {
        if (y[key] !== value) {
            return false;
        }
    }

    for (const [key, value] of Object.entries(y)) {
        if (x[key] !== value) {
            return false;
        }
    }

    return true;
}

function optionsChanged(options) {
    if (!tabOptions) {
        return;
    }

    const isDifferent = !objectsEqual(options, tabOptions);
    reloadButton.disabled = !isDifferent;
}

window.addEventListener("DOMContentLoaded", () => {
    bindBooleanOptions();
    addStorageChangeListener(optionsChanged);

    const optionsButton = document.getElementById("options-button");
    optionsButton.textContent = getI18nMessage("open_settings_page");
    optionsButton.addEventListener("click", () => openOptionsPage());

    reloadButton = document.getElementById("reload-button");
    reloadButton.textContent = getI18nMessage("action_reload");
    reloadButton.addEventListener("click", async () => {
        await reloadTab(activeTab.id);
        // TODO: wait for tab to load?
        setTimeout(() => {
            queryStatus((status) => {
                statusElement.textContent = getI18nMessage(status);
            });
        }, 1000);
    });

    const statusElement = document.getElementById("status");
    queryStatus((status) => {
        statusElement.textContent = getI18nMessage(status);
    });
});
