import * as utils from "./utils";
import type { LogLevel } from "ruffle-core";

import type { Duration } from "ruffle-core";

export interface Options {
    ruffleEnable: boolean;
    ignoreOptout: boolean;
    warnOnUnsupportedContent: boolean;
    logLevel: LogLevel;
    showSwfDownload: boolean;
    autostart: boolean;
    maxExecutionDuration: Duration;
}

interface OptionElement<T> {
    readonly input: Element;
    readonly label: HTMLLabelElement;
    value: T;
}

class CheckboxOption implements OptionElement<boolean> {
    constructor(
        private readonly checkbox: HTMLInputElement,
        readonly label: HTMLLabelElement
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

class MaxExecutionDuration_NumberInputOption implements OptionElement<Duration> {
    constructor(
        private readonly numberInput: HTMLInputElement,
        readonly label: HTMLLabelElement
    ) {}

    get input() {
        return this.numberInput;
    }

    get value() {
        return {
            secs: Math.max(1, Math.round(parseInt(this.numberInput.value))),
            nanos: 0,
        };
    }

    set value(value: Duration) {
        this.numberInput.value = "" + value.secs;
    }
}

class SelectOption implements OptionElement<string> {
    constructor(
        private readonly select: HTMLSelectElement,
        readonly label: HTMLLabelElement
    ) {}

    get input() {
        return this.select;
    }

    get value() {
        const index = this.select.selectedIndex;
        const option = this.select.options[index]!;
        return option.value;
    }

    set value(value: string) {
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

            if (input.id === "max_execution_duration") {

                return new MaxExecutionDuration_NumberInputOption(input, label);
            }
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
    onChange?: (options: Options) => void
): Promise<void> {
    const elements = findOptionElements();
    const options = await utils.getOptions();

    for (const [key, element] of elements.entries()) {
        // Bind initial value.
        element.value = options[key];

        // Prevent transition on load.
        // Method from https://stackoverflow.com/questions/11131875.
        element.label.classList.add("notransition");
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
