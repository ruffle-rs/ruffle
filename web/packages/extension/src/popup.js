import {
    getI18nMessage,
    sendMessageToTab,
    openOptionsPage,
    reloadTab,
    queryTabs,
} from "./utils";
import { bindBooleanOptions } from "./common";

let activeTab;
let savedOptions;
let tabOptions;

let statusIndicator;
let statusText;
let reloadButton;

// prettier-ignore
const STATUS_COLORS = {
    "status_init": "gray",
    "status_no_tab": "red",
    "status_tabs_error": "red",
    "status_message_init": "gray",
    "status_result_protected": "gray",
    "status_result_error": "red",
    "status_result_running": "green",
    "status_result_optout": "gray",
    "status_result_disabled": "gray",
};

async function queryTabStatus(listener) {
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
        response = await sendMessageToTab(activeTab.id, {});
    } catch (e) {
        listener("status_result_protected");
        reloadButton.disabled = true;
        return;
    }

    if (!response) {
        listener("status_result_error");
        return;
    }

    if (response.loaded) {
        listener("status_result_running");
    } else if (response.tabOptions.ruffleEnable) {
        listener("status_result_optout");
    } else {
        listener("status_result_disabled");
    }

    tabOptions = response.tabOptions;
    optionsChanged();
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

function optionsChanged() {
    if (!tabOptions) {
        return;
    }

    const isDifferent = !objectsEqual(savedOptions, tabOptions);
    reloadButton.disabled = !isDifferent;
}

function displayTabStatus() {
    queryTabStatus((status) => {
        statusIndicator.style.setProperty("--color", STATUS_COLORS[status]);
        statusText.textContent = getI18nMessage(status);
    });
}

window.addEventListener("DOMContentLoaded", () => {
    bindBooleanOptions((options) => {
        savedOptions = options;
        optionsChanged();
    });

    statusIndicator = document.getElementById("status-indicator");
    statusText = document.getElementById("status-text");

    const optionsButton = document.getElementById("options-button");
    optionsButton.textContent = getI18nMessage("open_settings_page");
    optionsButton.addEventListener("click", () => openOptionsPage());

    reloadButton = document.getElementById("reload-button");
    reloadButton.textContent = getI18nMessage("action_reload");
    reloadButton.addEventListener("click", async () => {
        await reloadTab(activeTab.id);
        // TODO: wait for tab to load?
        setTimeout(() => {
            displayTabStatus();
        }, 1000);
    });

    displayTabStatus();
});
