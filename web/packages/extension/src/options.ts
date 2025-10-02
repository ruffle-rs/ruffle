import * as utils from "./utils";
import { bindOptions, resetOptions } from "./common";
import { buildInfo } from "ruffle-core";

type SettingForm = {
    id: string;
    type: "text" | "number" | "select" | "switch";

    min?: number;
    max?: number;
    step?: number;

    placeholder?: string | number;

    label: string;
    description?: string;
    options?: SelectOption[];
};

type SelectOption = {
    text: string;
    value: string;
    defaultSelected?: boolean;
};

type SettingGroup = {
    settingsBasicPlayback: SettingForm[];
    settingsDisplay: SettingForm[];
    settingsPerformance: SettingForm[];
    settingsPlayerConfiguration: SettingForm[];
    settingsSecurityFeatures: SettingForm[];
    settingsUserInterface: SettingForm[];
    settingsFullscreenScaling: SettingForm[];
    settingMiscellaneous: SettingForm[];
};

const settingData: SettingGroup = {
    settingsBasicPlayback: [
        {
            id: "autoplay",
            type: "select",
            label: "Autoplay",
            description: "Controls the auto-play behaviour of Ruffle.",
            options: [
                { value: "auto", text: "Auto" },
                { value: "on", text: "On" },
                { value: "off", text: "Off" },
            ],
        },
        {
            id: "allowScriptAccess",
            type: "switch",
            label: "Allow Script Access",
            description:
                "Allow movie to interact with page through JavaScript.",
        },
        {
            id: "backgroundColor",
            type: "text",
            placeholder: "#FFFFFF",
            label: "Background Color",
            description: "HTML color for the background.",
        },
        {
            id: "letterbox",
            type: "select",
            label: "Letterbox",
            description:
                "Controls letterbox behavior when container size doesn't match movie size.",
            options: [
                {
                    value: "off",
                    text: "Off - The content will never be letterboxed.",
                },
                {
                    value: "fullscreen",
                    text: "Fullscreen - The content will only be letterboxed if the content is running fullscreen.",
                },
                {
                    value: "on",
                    text: "On - The content will always be letterboxed.",
                },
            ],
        },
        {
            id: "unmuteOverlay",
            type: "switch",
            label: "Unmute Overlay",
            description: "Show unmute overlay when player is muted.", // TODO: Update with better description to match switch
        },
        {
            id: "preloader",
            type: "select",
            label: "Preloader",
            description:
                "Whether or not to show a splash screen before the SWF has loaded.",
        },
        {
            id: "parameters",
            type: "text",
            placeholder: "key1=value1&key2=value2",
            label: "Parameters Flashvars",
            description:
                "Values that may be passed to and loaded by the movie.",
        },
    ],
    settingsDisplay: [
        {
            id: "wmode",
            type: "select",
            label: "Window Mode",
            description: "Controls how Ruffle is layered with other content.",
            options: [
                { value: "window", text: "Window - Default browser layering" },
                {
                    value: "opaque",
                    text: "Opaque - Layers with HTML, opaque background",
                },
                {
                    value: "transparent",
                    text: "Transparent - Layers with HTML, transparent background",
                },
                {
                    value: "direct",
                    text: "Direct - Hardware acceleration same as Opaque",
                },
                { value: "gpu", text: "GPU - Direct rendering same as Opaque" },
            ],
        },
        {
            id: "scale",
            type: "select",
            label: "Scale Mode",
            description:
                "Equivalent to Stage.scaleMode (e.g., showAll, noBorder, exactFit, noScale).",
            options: [
                {
                    value: "exactFit",
                    text: "Exact Fit - Stretches to fill container, aspect ratio not preserved",
                },
                {
                    value: "noBorder",
                    text: "No Border - Fills container, preserves aspect ratio, edges may be cropped",
                },
                {
                    value: "noScale",
                    text: "No Scale - No scaling, content uses container size may be clipped",
                },
                {
                    value: "showAll",
                    text: "Show All - Scales to fit container, preserves aspect ratio letterboxed if needed",
                },
            ],
        },
        {
            id: "salign",
            type: "select",
            label: "Stage Alignment",
            description:
                "Equivalent to Stage.align. Choose how content is anchored in the container.",
            options: [
                { value: "C", text: "Center" },
                { value: "T", text: "Top" },
                { value: "B", text: "Bottom" },
                { value: "L", text: "Left" },
                { value: "R", text: "Right" },
                { value: "TL", text: "Top Left" },
                { value: "TR", text: "Top Right" },
                { value: "BL", text: "Bottom Left" },
                { value: "BR", text: "Bottom Right" },
            ],
        },
        {
            // TODO: Select
            id: "quality",
            type: "text",
            label: "Quality",
            description:
                "Equivalent to Stage.quality (e.g., low, medium, high, best).",
        },
        {
            id: "scrollingBehavior",
            type: "select",
            label: "Scrolling Behavior",
            description:
                "Defines the scrolling behavior of Flash content on the web page.",
            options: [
                {
                    value: "smart",
                    text: "Smart - Scroll the page only when the Flash content hasn't handled the scroll.",
                },
                { value: "never", text: "Never - Never scroll the page." },
                { value: "always", text: "Always - Always scroll the page." },
            ],
        },
    ],
    settingsPerformance: [
        {
            id: "preferredRenderer",
            type: "select",
            label: "Preferred Renderer",
            description: "Preferred render backend for this site.",
            options: [
                { value: "Automatic", text: "Automatic" },
                { value: "webgpu", text: "WebGPU" },
                { value: "wgpu-webgl", text: "wgpu via WebGL" },
                { value: "webgl", text: "WebGL" },
                { value: "canvas", text: "Canvas" },
            ],
        },
        {
            id: "frameRate",
            type: "number",
            min: 1,
            max: 120,
            step: 1,
            label: "Frame Rate Override",
            description:
                "Lock player's frame rate overrides movies frame rate.",
        },
        {
            id: "maxExecutionDuration",
            type: "number",
            min: 1,
            max: 15,
            step: 1,
            placeholder: 15,
            label: "Max Script Execution seconds",
            description: "Maximum time a script can run before being disabled.",
        },
    ],
    settingsPlayerConfiguration: [
        {
            id: "playerVersion",
            type: "number",
            min: 1,
            max: 32,
            step: 1,
            label: "Player Version",
            description: "Flash player version to report to the movie.",
        },
        {
            id: "playerRuntime",
            type: "select",
            label: "Player Runtime",
            description: "Runtime environment to emulate.",
            options: [
                { value: "flashPlayer", text: "Flash Player" },
                { value: "air", text: "Adobe AIR" },
            ],
        },
        {
            id: "logLevel",
            type: "select",
            label: "Log Level",
            description: "Console logging level for this site.",
            options: [
                { value: "error", text: "Error" },
                { value: "warn", text: "Warning" },
                { value: "info", text: "Info" },
                { value: "debug", text: "Debug" },
                { value: "trace", text: "Trace" },
            ],
        },
        {
            id: "publicPath",
            type: "text",
            placeholder: "/path/to/ruffle/",
            label: "Public Path",
            description:
                "The URL at which Ruffle can load its extra files i.e. .wasm.",
        },
        {
            id: "polyfills",
            type: "switch",
            label: "Polyfills",
            description:
                "Enable polyfills on the page for legacy Flash content.",
        },
        {
            id: "fontSources",
            type: "text",
            placeholder: "/path/to/font.swf,/path/to/font2.swf",
            label: "Font Sources",
            description: "List of font URLs separated by comma to load.",
        },
        // TODO: Add via dynamic inputs dynamic
        /* {
            id: 'defaultFonts',
            type: 'text',
            label: '...',
            description: 'Names of fonts to use for each default Flash device font.',
        }, */
    ],
    settingsSecurityFeatures: [
        {
            id: "allowNetworking",
            type: "select",
            label: "Network Access Mode",
            description: "Which flash networking APIs may be accessed.",
            options: [
                { value: "all", text: "All" },
                { value: "internal", text: "Internal" },
                { value: "none", text: "None" },
            ],
        },
        {
            id: "openUrlMode",
            type: "select",
            label: "Open URL Mode",
            description: "Handling mode for links opening new websites.",
            options: [
                { value: "allow", text: "Allow" },
                { value: "confirm", text: "Confirm" },
                { value: "deny", text: "Deny" },
            ],
        },
        {
            id: "upgradeToHttps",
            type: "switch",
            label: "Upgrade to HTTPS",
            description: "Auto-upgrade embedded HTTP URLs to HTTPS.",
        },
        {
            id: "compatibilityRules",
            type: "switch",
            label: "Compatibility Rules",
            description: "Enable Ruffle's built-in compatibility rules.",
        },
        {
            id: "favorFlash",
            type: "switch",
            label: "Favor Flash Player",
            description: "Prefer real Adobe Flash Player if available.",
        },
        // I am not sure if this can be done via UI
        /* {
            id: 'openInNewTab',
            type: 'text',
            placeholder: '...',
            label: 'Open in New Tab',
            description: 'Function to open content in a new tab.',
        }, */
        // TODO: Add more inputs dynamic or use Array? e.g. [{'port': 80, 'host': '0.0.0.0'..}]
        /* {
            id: 'credentialsocketProxyAllowList',
            type: 'text',
            placeholder: '...',
            label: 'Socket Proxy',
            description: 'Array of SocketProxy objects for socket connections.',
        }, */
        {
            id: "credentialAllowList",
            type: "text",
            placeholder: "https://example.org,https://example2.org",
            label: "Credential Allow List",
            description:
                "List of origins separated by comma to which credentials can be sent.",
        },
        {
            id: "gamepadButtonMapping",
            type: "text",
            placeholder: "dpad-up:38,dpad-down:40",
            label: "Gamepad Button Mapping",
            description:
                "Mapping of gamepad buttons to ActionScript key codes.",
        },
        /* { I am not sure if this can be done via UI without adding more inputs and validations
            id: 'urlRewriteRules',
            type: 'text',
            label: 'URL Rewrite Rules',
            description: 'Set of rules that rewrite URLs in network requests and links.',
        }, */
    ],
    settingsUserInterface: [
        {
            id: "contextMenu",
            type: "select",
            label: "Context Menu",
            description:
                "Show context menu when right-clicking or long-pressing.",
            options: [
                { value: "on", text: "On" },
                { value: "rightClickOnly", text: "Right Click Only" },
                { value: "off", text: "Off" },
            ],
        },
        {
            id: "showSwfDownload",
            type: "switch",
            label: "Show SWF Download",
            description: "Add SWF download option to context menu.",
        },
        {
            id: "menu",
            type: "switch",
            label: "Built-in Menu Items",
            description: "Equivalent to Stage.showMenu.", // TODO: Update with better description to match switch
        },
        {
            id: "splashScreen",
            type: "switch",
            label: "Splash Screen",
            description: "Show splash screen before SWF loads.", // TODO: Update with better description to match switch
        },
    ],
    settingsFullscreenScaling: [
        {
            id: "allowFullscreen",
            type: "switch",
            label: "Allow Fullscreen",
            description: "Allow Stage's displayState to be changed.",
        },
        {
            id: "forceAlign",
            type: "switch",
            label: "Force Alignment",
            description: "Prevent movies from changing stage alignment.",
        },
        {
            id: "forceScale",
            type: "switch",
            label: "Force Scale",
            description: "Prevent movies from changing stage scale mode.",
        },
        {
            id: "fullScreenAspectRatio",
            type: "text",
            label: "Fullscreen Aspect Ratio",
            description: "Controls orientation on mobile in fullscreen mode.",
        },
    ],
    settingMiscellaneous: [
        {
            id: "base",
            type: "text",
            label: "Base URL",
            description:
                "Base directory/URL for resolving relative paths in SWF.",
        },
    ],
};

function addElement(settingForm: SettingForm): HTMLElement {
    const settingsOption = document.createElement("div");
    settingsOption.classList.add("settings-option");

    const settingsOptionToggle = document.createElement("input");
    settingsOptionToggle.classList.add("settings-option-toggle");
    settingsOptionToggle.type = "checkbox";
    settingsOptionToggle.dataset["optionId"] = settingForm.id;

    settingsOption.appendChild(settingsOptionToggle);

    const settingsOptionControl = document.createElement("div");
    settingsOptionControl.classList.add("settings-option-control");
    settingsOptionControl.id = `control-${settingForm.id}`;

    settingsOption.appendChild(settingsOptionControl);

    const formElement = document.createElement("div");
    formElement.classList.add("form-element");

    settingsOptionControl.appendChild(formElement);

    const formGroup = document.createElement("div");
    formGroup.classList.add("form-group");

    formElement.appendChild(formGroup);

    const formLabel = document.createElement("label");
    formLabel.classList.add("form-label");
    formLabel.htmlFor = `setting-${settingForm.id}`;
    formLabel.innerText = settingForm.label;

    formGroup.appendChild(formLabel);

    if (settingForm.description) {
        const smallDescription = document.createElement("small");
        smallDescription.innerText = settingForm.description;

        formGroup.appendChild(smallDescription);
    }

    switch (settingForm.type) {
        case "text":
            {
                const formTypeInput = document.createElement("input");
                formTypeInput.id = `setting-${settingForm.id}`;
                formTypeInput.classList.add("form-type-text");

                if (settingForm.placeholder) {
                    formTypeInput.placeholder =
                        settingForm.placeholder.toString();
                }

                formElement.appendChild(formTypeInput);
            }
            break;
        case "number":
            {
                const formTypeNumber = document.createElement("input");
                formTypeNumber.id = `setting-${settingForm.id}`;
                formTypeNumber.classList.add("form-type-number");

                if (settingForm.placeholder) {
                    formTypeNumber.placeholder =
                        settingForm.placeholder.toString();
                }

                formElement.appendChild(formTypeNumber);
            }
            break;
        case "select":
            {
                const formTypeSelect = document.createElement("select");
                formTypeSelect.id = `setting-${settingForm.id}`;
                formTypeSelect.classList.add("form-type-select");

                if (settingForm.options) {
                    for (const opt of settingForm.options) {
                        formTypeSelect.add(
                            new Option(
                                opt.text,
                                opt.value,
                                opt.defaultSelected,
                            ),
                        );
                    }
                }

                formElement.appendChild(formTypeSelect);
            }
            break;
        case "switch":
            {
                const formTypeSwitch = document.createElement("div");
                formTypeSwitch.classList.add("form-type-switch");

                const formTypeSwitchCheckbox = document.createElement("input");
                formTypeSwitchCheckbox.id = `setting-${settingForm.id}`;
                formTypeSwitchCheckbox.type = "checkbox";

                formTypeSwitch.appendChild(formTypeSwitchCheckbox);

                const formTypeSwitchSlider = document.createElement("div");
                formTypeSwitchSlider.classList.add("slider");

                formTypeSwitch.appendChild(formTypeSwitchSlider);

                formElement.appendChild(formTypeSwitch);
            }
            break;
    }

    return settingsOption;
}

function addSettingGroup(
    targetGroup: HTMLElement,
    settingsForm: SettingForm[],
) {
    const htmlElements: HTMLElement[] = [];

    for (const settingForm of settingsForm) {
        htmlElements.push(addElement(settingForm));
    }

    console.log(...htmlElements);

    targetGroup.append(...htmlElements);
}

window.addEventListener("DOMContentLoaded", async () => {
    addSettingGroup(
        document.getElementById("settings-basic-playback")!,
        settingData.settingsBasicPlayback,
    );

    addSettingGroup(
        document.getElementById("settings-display")!,
        settingData.settingsDisplay,
    );

    addSettingGroup(
        document.getElementById("settings-performance")!,
        settingData.settingsPerformance,
    );

    addSettingGroup(
        document.getElementById("settings-player-configuration")!,
        settingData.settingsPlayerConfiguration,
    );

    addSettingGroup(
        document.getElementById("settings-security-features")!,
        settingData.settingsSecurityFeatures,
    );

    addSettingGroup(
        document.getElementById("settings-user-interface")!,
        settingData.settingsUserInterface,
    );

    addSettingGroup(
        document.getElementById("settings-fullscreen-scaling")!,
        settingData.settingsFullscreenScaling,
    );

    addSettingGroup(
        document.getElementById("settings-miscellaneous")!,
        settingData.settingMiscellaneous,
    );

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
