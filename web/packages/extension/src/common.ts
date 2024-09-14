import * as utils from "./utils";
import type { BaseLoadOptions } from "ruffle-core";

export interface Options extends BaseLoadOptions {
    ruffleEnable: boolean;
    ignoreOptout: boolean;
    autostart: boolean;
    showReloadButton: boolean;
    swfTakeover: boolean;
}

interface OptionElement<T> {
    readonly input: Element;
    readonly label: HTMLLabelElement;
    value: T;
}

class CheckboxOption implements OptionElement<boolean> {
    constructor(
        private readonly checkbox: HTMLInputElement,
        readonly label: HTMLLabelElement,
    ) {}

    get input() {
        return this.checkbox;
    }

    get value() {
        return this.checkbox.checked;
    }

    set value(value: boolean) {
        this.checkbox.checked = value;
    }
}

class NumberOption implements OptionElement<number | null> {
    constructor(
        private readonly numberInput: HTMLInputElement,
        readonly label: HTMLLabelElement,
    ) {
        this.numberInput.addEventListener("input", () => {
            this.numberInput.reportValidity();
        });
    }

    get input() {
        return this.numberInput;
    }

    get value(): number | null {
        const ni = this.numberInput;
        const num = ni.valueAsNumber;
        if (Number.isNaN(num)) {
            return null;
        }
        if (ni.min) {
            const min = Number(ni.min);
            if (min > num) {
                return min;
            }
        }
        if (ni.max) {
            const max = Number(ni.max);
            if (num > max) {
                return max;
            }
        }
        return num;
    }

    set value(value: number | null) {
        this.numberInput.value = value?.toString() ?? "";
    }
}

class SelectOption implements OptionElement<string | null> {
    constructor(
        private readonly select: HTMLSelectElement,
        readonly label: HTMLLabelElement,
    ) {
        // Localize each `option`, if relevant.
        Array.prototype.forEach.call(select.options, (option) => {
            if (option.hasAttribute("id")) {
                const message = utils.i18n.getMessage(`settings_${option.id}`);
                if (message) {
                    option.textContent = message;
                }
            }
        });
    }

    get input() {
        return this.select;
    }

    get value() {
        const index = this.select.selectedIndex;
        const option = this.select.options[index]!;
        // Convert the empty string to `null`.
        return option.value || null;
    }

    set value(value: string | null) {
        // Convert `null` to the empty string.
        value ??= "";
        const options = Array.from(this.select.options);
        const index = options.findIndex((option) => option.value === value);
        this.select.selectedIndex = index;
    }
}

function getElement(option: Element): OptionElement<unknown> {
    const label = option.getElementsByTagName("label")[0]!;

    const [input] = option.getElementsByTagName("input");
    if (input) {
        if (input.type === "checkbox") {
            return new CheckboxOption(input, label);
        }

        if (input.type === "number") {
            return new NumberOption(input, label);
        }
    }

    const [select] = option.getElementsByTagName("select");
    if (select) {
        return new SelectOption(select, label);
    }

    throw new Error("Unknown option element");
}

function findOptionElements() {
    const camelize = (s: string) =>
        s.replace(/[^a-z\d](.)/gi, (_, char) => char.toUpperCase());

    const elements = new Map<keyof Options, OptionElement<unknown>>();
    for (const option of document.getElementsByClassName("option")) {
        const element = getElement(option);
        const key = camelize(element.input.id) as keyof Options;
        elements.set(key, element);
    }
    return elements;
}

export async function bindOptions(
    onChange?: (options: Options) => void,
): Promise<void> {
    const elements = findOptionElements();
    const options = await utils.getOptions();

    for (const [key, element] of elements.entries()) {
        // Bind initial value.
        element.value = options[key];

        // Prevent transition on load.
        // Method from https://stackoverflow.com/questions/11131875.
        element.label.classList.add("notransition");
        // eslint-disable-next-line @typescript-eslint/no-unused-expressions
        element.label.offsetHeight; // Trigger a reflow, flushing the CSS changes.
        element.label.classList.remove("notransition");

        // Localize label.
        const message = utils.i18n.getMessage(`settings_${element.input.id}`);
        if (message) {
            element.label.textContent = message;
        }

        // Listen for user input.
        element.input.addEventListener("change", () => {
            const value = element.value;
            options[key] = value as never;
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
            element.value = option.newValue;
            options[key as keyof Options] = option.newValue as never;
        }

        if (onChange) {
            onChange(options);
        }
    });

    if (onChange) {
        onChange(options);
    }
}

export async function resetOptions(): Promise<void> {
    // This setting is consistent for the browser in use and should not change
    const data = await utils.storage.sync.get({
        responseHeadersUnsupported: false,
    });
    await utils.storage.sync.clear();
    if (data["responseHeadersUnsupported"]) {
        utils.storage.sync.set({ responseHeadersUnsupported: true });
    }
}
