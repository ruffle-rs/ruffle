import * as utils from "./utils";

function camelize(s: string) {
    return s.replace(/[^a-z\d](.)/gi, (_, char) => char.toUpperCase());
}

export interface Options {
    ruffleEnable: boolean;
    ignoreOptout: boolean;
}

function getBooleanElements() {
    const elements: Record<
        string,
        { option: Element; checkbox: HTMLInputElement; label: HTMLLabelElement }
    > = {};
    for (const option of document.getElementsByClassName("option")) {
        const [checkbox] = option.getElementsByTagName("input");
        if (checkbox.type !== "checkbox") {
            continue;
        }
        const [label] = option.getElementsByTagName("label");
        const key = camelize(checkbox.id);
        elements[key] = { option, checkbox, label };
    }
    return elements;
}

export async function bindBooleanOptions(
    onChange?: (options: Options) => void
): Promise<void> {
    const elements = getBooleanElements();

    // Bind initial values.
    const options = await utils.getOptions(Object.keys(elements));
    for (const [key, value] of Object.entries(options)) {
        elements[key].checkbox.checked = value;
    }

    for (const [key, { checkbox, label }] of Object.entries(elements)) {
        // TODO: click/change/input?
        checkbox.addEventListener("click", () => {
            const value = checkbox.checked;
            options[key as keyof Options] = value;
            utils.storage.sync.set({ [key]: value });
        });

        label.textContent = utils.i18n.getMessage(`settings_${checkbox.id}`);

        // Prevent transition on load.
        // Method from https://stackoverflow.com/questions/11131875.
        label.classList.add("notransition");
        label.offsetHeight; // Trigger a reflow, flushing the CSS changes.
        label.classList.remove("notransition");
    }

    // Listen for future changes.
    utils.storage.onChanged.addListener((changes, namespace) => {
        if (namespace !== "sync") {
            return;
        }

        for (const [key, option] of Object.entries(changes)) {
            if (!elements[key]) {
                continue;
            }
            elements[key].checkbox.checked = option.newValue;
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
