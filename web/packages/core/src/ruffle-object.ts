import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    FLASH_ACTIVEX_CLASSID,
    isSwfFilename,
    RufflePlayer,
} from "./ruffle-player";
import { registerElement } from "./register-element";
import { URLLoadOptions } from "./load-options";

/**
 * Find and return the first value in obj with the given key.
 * Many Flash params were case insensitive, so we use this when checking for them.
 *
 * @param obj Object to check
 * @param key Key to find
 * @param defaultValue Value if not found
 * @returns Value if found, else [[defaultValue]]
 */
function findCaseInsensitive(
    obj: { [key: string]: string | null },
    key: string,
    defaultValue: string | null
): string | null {
    key = key.toLowerCase();
    for (const k in obj) {
        if (Object.hasOwnProperty.call(obj, k) && key === k.toLowerCase()) {
            return obj[k];
        }
    }
    return defaultValue;
}

/**
 * Returns all flash params ([[HTMLParamElement]]) that are for the given object.
 *
 * @param elem Element to check.
 * @returns A record of every parameter.
 */
function paramsOf(elem: HTMLElement): Record<string, string> {
    const params: Record<string, string> = {};

    for (const param of elem.children) {
        if (param instanceof HTMLParamElement) {
            const key = param.attributes.getNamedItem("name")?.value;
            const value = param.attributes.getNamedItem("value")?.value;
            if (key && value) {
                params[key] = value;
            }
        }
    }

    return params;
}

/**
 * A polyfill html element.
 *
 * This specific class tries to polyfill existing `<object>` tags,
 * and should not be used. Prefer [[RufflePlayer]] instead.
 *
 * @internal
 */
export class RuffleObject extends RufflePlayer {
    private params: Record<string, string> = {};

    /**
     * Constructs a new Ruffle flash player for insertion onto the page.
     *
     * This specific class tries to polyfill existing `<object>` tags,
     * and should not be used. Prefer [[RufflePlayer]] instead.
     */
    constructor() {
        super();
    }

    /**
     * @ignore
     * @internal
     */
    connectedCallback(): void {
        super.connectedCallback();

        this.params = paramsOf(this);

        const allowScriptAccess = findCaseInsensitive(
            this.params,
            "allowScriptAccess",
            "sameDomain"
        );
        let url = null;

        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }

        const parameters = findCaseInsensitive(
            this.params,
            "flashvars",
            this.getAttribute("flashvars")
        );

        if (url) {
            this.allowScriptAccess = !!(
                allowScriptAccess &&
                (allowScriptAccess.toLowerCase() === "always" ||
                    (allowScriptAccess.toLowerCase() === "samedomain" &&
                        new URL(window.location.href).origin ===
                            new URL(url, window.location.href).origin))
            );

            //Kick off the SWF download.
            const options: URLLoadOptions = { url };
            if (parameters) {
                options.parameters = parameters;
            }
            this.load(options);
        }
    }

    protected debugPlayerInfo(): string {
        let errorText = super.debugPlayerInfo();
        errorText += "Player type: Object\n";

        let url = null;

        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }
        errorText += `SWF URL: ${url}\n`;

        Object.keys(this.params).forEach((key) => {
            errorText += `Param ${key}: ${this.params[key]}\n`;
        });

        Object.keys(this.attributes).forEach((key) => {
            errorText += `Attribute ${key}: ${
                this.attributes.getNamedItem(key)?.value
            }\n`;
        });

        return errorText;
    }

    /**
     * Polyfill of HTMLObjectElement.
     *
     * @ignore
     * @internal
     */
    get data(): string | null {
        return this.getAttribute("data");
    }

    /**
     * Polyfill of HTMLObjectElement.
     *
     * @ignore
     * @internal
     */
    set data(href: string | null) {
        if (href != undefined) {
            const attr = document.createAttribute("data");
            attr.value = href;
            this.attributes.setNamedItem(attr);
        } else {
            this.attributes.removeNamedItem("data");
        }
    }

    /**
     * Checks if the given element may be polyfilled with this one.
     *
     * @param elem Element to check.
     * @returns True if the element looks like a flash object.
     */
    static isInterdictable(elem: HTMLElement): boolean {
        const data = elem.attributes.getNamedItem("data")?.value.toLowerCase();
        if (!data) {
            let hasMovie = false;
            const params = elem.getElementsByTagName("param");
            for (let i = 0; i < params.length; i++) {
                if (params[i].name == "movie" && params[i].value) {
                    hasMovie = true;
                }
            }
            if (!hasMovie) {
                return false;
            }
        }

        const type = elem.attributes.getNamedItem("type")?.value.toLowerCase();
        const classid = elem.attributes
            .getNamedItem("classid")
            ?.value.toLowerCase();
        if (
            type === FLASH_MIMETYPE.toLowerCase() ||
            type === FUTURESPLASH_MIMETYPE.toLowerCase() ||
            type === FLASH7_AND_8_MIMETYPE.toLowerCase() ||
            type === FLASH_MOVIE_MIMETYPE.toLowerCase()
        ) {
            return true;
        } else if (classid === FLASH_ACTIVEX_CLASSID.toLowerCase()) {
            return true;
        } else if (
            (type == null || type === "") &&
            (classid == null || classid === "")
        ) {
            const params = paramsOf(elem);
            if (data && isSwfFilename(data)) {
                return true;
            } else if (params && params.movie && isSwfFilename(params.movie)) {
                return true;
            }
        }

        return false;
    }

    /**
     * Creates a RuffleObject that will polyfill and replace the given element.
     *
     * @param elem Element to replace.
     * @returns Created RuffleObject.
     */
    static fromNativeObjectElement(elem: HTMLElement): RuffleObject {
        const externalName = registerElement("ruffle-object", RuffleObject);
        const ruffleObj: RuffleObject = <RuffleObject>(
            document.createElement(externalName)
        );
        ruffleObj.copyElement(elem);

        return ruffleObj;
    }
}
