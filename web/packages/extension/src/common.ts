import * as utils from "./utils";

export interface Options {
    ruffleEnable: boolean;
    ignoreOptout: boolean;
}

function getBooleanElements() {
    const camelize = (s: string) =>
        s.replace(/[^a-z\d](.)/gi, (_, char) => char.toUpperCase());

    const elements = new Map<
        keyof Options,
        { option: Element; checkbox: HTMLInputElement; label: HTMLLabelElement }
    >();
    for (const option of document.getElementsByClassName("option")) {
        const [checkbox] = option.getElementsByTagName("input");
        if (checkbox.type !== "checkbox") {
            continue;
        }
        const [label] = option.getElementsByTagName("label");
        const key = camelize(checkbox.id) as keyof Options;
        elements.set(key, { option, checkbox, label });
    }
    return elements;
}

export async function bindBooleanOptions(
    onChange?: (options: Options) => void
): Promise<void> {
    const elements = getBooleanElements();
    const options = await utils.getOptions(Array.from(elements.keys()));

    for (const [key, { checkbox, label }] of elements.entries()) {
        // Bind initial value.
        checkbox.checked = options[key];

        // Prevent transition on load.
        // Method from https://stackoverflow.com/questions/11131875.
        label.classList.add("notransition");
        label.offsetHeight; // Trigger a reflow, flushing the CSS changes.
        label.classList.remove("notransition");

        // Localize label.
        const message = utils.i18n.getMessage(`settings_${checkbox.id}`);
        if (message) {
            label.textContent = message;
        }

        // Listen for user input.
        checkbox.addEventListener("change", () => {
            const value = checkbox.checked;
            options[key] = value;
            utils.storage.sync.set({ [key]: value });
        });
    }

    // Listen for future changes.
    utils.storage.onChanged.addListener((changes, namespace) => {
        if (namespace !== "sync") {
            return;
        }

        for (const [key, option] of Object.entries(changes)) {
            const element = elements.get(key as keyof Options);
            if (!element) {
                continue;
            }
            element.checkbox.checked = option.newValue;
            options[key as keyof Options] = option.newValue;
        }

        if (onChange) {
            onChange(options);
        }
    });

    if (onChange) {
        onChange(options);
    }
}
