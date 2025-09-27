import * as utils from "./utils";
import { bindOptions, resetOptions } from "./common";
import { buildInfo } from "ruffle-core";

window.addEventListener("DOMContentLoaded", async () => {
    const data = await utils.storage.sync.get({
        responseHeadersUnsupported: false,
    });
    if (data["responseHeadersUnsupported"]) {
        document
            .getElementById("swf_takeover")!
            .parentElement!.classList.add("hidden");
    }
    document.title = utils.i18n.getMessage("settings_page");
    {
        const vt = document.getElementById("version-text")!;
        vt.textContent = buildInfo.versionName;
    }
    {
        const ao = document.getElementById("advanced-options")!;
        ao.textContent = utils.i18n.getMessage("settings_advanced_options");
    }
    {
        const rs = document.getElementById("reset-settings")!;
        rs.textContent = utils.i18n.getMessage("settings_reset");
        rs.addEventListener("click", async () => {
            if (confirm(utils.i18n.getMessage("settings_reset_confirm"))) {
                await resetOptions();
                window.location.reload();
            }
        });
    }

    const modal = document.getElementById("site-settings-modal")!;
    const addNewBtn = document.getElementById("site-entry-new")!;
    const closeBtns = document.querySelectorAll(
        ".modal-close-btn, #modal-cancel-btn",
    );

    const openModal = () => {
        modal.style.display = "flex";
        document.body.classList.add("modal-open");
    };

    const closeModal = () => {
        modal.style.display = "none";
        document.body.classList.remove("modal-open");
    };

    addNewBtn.addEventListener("click", openModal);

    closeBtns.forEach((btn) => btn.addEventListener("click", closeModal));

    document.querySelectorAll(".edit-site-btn").forEach((btn) => {
        btn.addEventListener("click", openModal);
    });

    document.querySelectorAll(".settings-option").forEach((option) => {
        const switchEl = option.querySelector<HTMLInputElement>(
            ".settings-option-toggle",
        )!;
        const controlId = switchEl.dataset["optionId"];
        const controlContainer = document.getElementById(
            `control-${controlId}`,
        );

        if (!controlContainer) {
            console.warn(`Element with id control-${controlId} not found.`);
            return;
        }

        const toggleControl = () => {
            if (switchEl.checked) {
                controlContainer.classList.remove("settings-option-disabled");
            } else {
                controlContainer.classList.add("settings-option-disabled");
            }
        };

        switchEl.addEventListener("change", toggleControl);

        toggleControl();
    });

    bindOptions();
});
