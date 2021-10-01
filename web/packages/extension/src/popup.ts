import * as utils from "./utils";
import { Options, bindOptions } from "./common";

let activeTab: chrome.tabs.Tab | browser.tabs.Tab;
let savedOptions: Options;
let tabOptions: Options;

let statusIndicator: HTMLDivElement;
let statusText: HTMLSpanElement;
let reloadButton: HTMLButtonElement;

// prettier-ignore
const STATUS_COLORS = {
    "status_init": "gray",
    "status_no_tabs": "red",
    "status_tabs_error": "red",
    "status_message_init": "gray",
    "status_result_protected": "gray",
    "status_result_error": "red",
    "status_result_running": "green",
    "status_result_optout": "gray",
    "status_result_disabled": "gray",
};

async function queryTabStatus(
    listener: (status: keyof typeof STATUS_COLORS) => void
) {
    listener("status_init");

    let tabs: chrome.tabs.Tab[] | browser.tabs.Tab[];
    try {
        tabs = await utils.tabs.query({
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
        response = await utils.tabs.sendMessage(activeTab.id!, {
            type: "ping",
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

    tabOptions = response.tabOptions;

    if (response.loaded) {
        listener("status_result_running");
    } else if (tabOptions.ruffleEnable) {
        listener("status_result_optout");
    } else {
        listener("status_result_disabled");
    }

    optionsChanged();
}

function objectsEqual<T>(x: T, y: T) {
    for (const [key, value] of Object.entries(x)) {
        if (y[key as keyof T] !== value) {
            return false;
        }
    }

    for (const [key, value] of Object.entries(y)) {
        if (x[key as keyof T] !== value) {
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
        statusText.textContent = utils.i18n.getMessage(status);
    });
}

window.addEventListener("DOMContentLoaded", () => {
    bindOptions((options) => {
        savedOptions = options;
        optionsChanged();
    });

    statusIndicator = document.getElementById(
        "status-indicator"
    ) as HTMLDivElement;
    statusText = document.getElementById("status-text") as HTMLSpanElement;

    const optionsButton = document.getElementById(
        "options-button"
    ) as HTMLButtonElement;
    optionsButton.textContent = utils.i18n.getMessage("open_settings_page");
    optionsButton.addEventListener("click", () => utils.openOptionsPage());

    reloadButton = document.getElementById(
        "reload-button"
    ) as HTMLButtonElement;
    reloadButton.textContent = utils.i18n.getMessage("action_reload");
    reloadButton.addEventListener("click", async () => {
        await utils.tabs.reload(activeTab.id!);
        // TODO: Wait for tab to load?
        setTimeout(() => {
            displayTabStatus();
        }, 1000);
    });

    displayTabStatus();
});
