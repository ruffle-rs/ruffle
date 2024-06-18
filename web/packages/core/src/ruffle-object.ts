import {
    isFallbackElement,
    isYoutubeFlashSource,
    workaroundYoutubeMixedContent,
    RufflePlayer,
    getPolyfillOptions,
} from "./ruffle-player";
import { FLASH_ACTIVEX_CLASSID } from "./flash-identifiers";
import { registerElement } from "./register-element";
import { RuffleEmbed } from "./ruffle-embed";
import { isSwf } from "./swf-utils";

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
    obj: Record<string, string>,
    key: string,
    defaultValue: string | null,
): string | null {
    key = key.toLowerCase();
    for (const [k, value] of Object.entries(obj)) {
        if (k.toLowerCase() === key) {
            return value;
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
function paramsOf(elem: Element): Record<string, string> {
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
    override connectedCallback(): void {
        super.connectedCallback();

        this.params = paramsOf(this);

        let url = null;
        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params["movie"]) {
            url = this.params["movie"];
        }

        if (url) {
            // Get the configuration options that have been overwritten for this movie.
            const attributeCheckOptions = [
                "allowNetworking",
                "base",
                "bgcolor",
                "flashvars",
            ];
            const getOptionString = (optionName: string) =>
                findCaseInsensitive(
                    this.params,
                    optionName,
                    attributeCheckOptions.includes(optionName)
                        ? this.getAttribute(optionName)
                        : null,
                );
            const options = getPolyfillOptions(url, getOptionString);

            // Kick off the SWF download.
            this.load(options, true);
        }
    }

    protected override debugPlayerInfo(): string {
        let result = "Player type: Object\n";

        let url = null;
        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params["movie"]) {
            url = this.params["movie"];
        }
        result += `SWF URL: ${url}\n`;

        Object.keys(this.params).forEach((key) => {
            result += `Param ${key}: ${this.params[key]}\n`;
        });

        Object.keys(this.attributes).forEach((key) => {
            result += `Attribute ${key}: ${
                this.attributes.getNamedItem(key)?.value
            }\n`;
        });

        return result;
    }

    /**
     * Polyfill of HTMLObjectElement.
     *
     * @ignore
     * @internal
     */
    override get nodeName(): string {
        return "OBJECT";
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
        if (href) {
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
     * @returns True if the element looks like a Flash object.
     */
    static isInterdictable(elem: Element): boolean {
        // Don't polyfill if the element is inside a specific node.
        if (isFallbackElement(elem)) {
            return false;
        }

        // Don't polyfill if there's already a <ruffle-object> or a <ruffle-embed> inside the <object>.
        if (
            elem.getElementsByTagName("ruffle-object").length > 0 ||
            elem.getElementsByTagName("ruffle-embed").length > 0
        ) {
            return false;
        }

        const data = elem.attributes.getNamedItem("data")?.value.toLowerCase();
        const type = elem.attributes.getNamedItem("type")?.value ?? null;
        const params = paramsOf(elem);

        // Check for SWF file.
        let filename;
        if (data) {
            // Don't polyfill when the file is a YouTube Flash source.
            if (isYoutubeFlashSource(data)) {
                // Workaround YouTube mixed content; this isn't what browsers do automatically, but while we're here, we may as well.
                workaroundYoutubeMixedContent(elem, "data");
                return false;
            }
            filename = data;
        } else if (params && params["movie"]) {
            // Don't polyfill when the file is a YouTube Flash source.
            if (isYoutubeFlashSource(params["movie"])) {
                // Workaround YouTube mixed content; this isn't what browsers do automatically, but while we're here, we may as well.
                const movieElem = elem.querySelector("param[name='movie']");
                if (movieElem) {
                    workaroundYoutubeMixedContent(movieElem, "value");
                    // The data attribute needs to be set for the re-fetch to happen.
                    // It also needs to be set on Firefox for the YouTube object rewrite to work, regardless of mixed content.
                    const movieSrc = movieElem.getAttribute("value");
                    if (movieSrc) {
                        elem.setAttribute("data", movieSrc);
                    }
                }
                return false;
            }
            filename = params["movie"];
        } else {
            // Don't polyfill when no file is specified.
            return false;
        }

        // Check ActiveX class ID.
        const classid = elem.attributes
            .getNamedItem("classid")
            ?.value.toLowerCase();
        if (classid === FLASH_ACTIVEX_CLASSID.toLowerCase()) {
            // classid is an old-IE style embed that would not work on modern browsers.
            // Often there will be an <embed> inside the <object> that would take precedence.
            // Only polyfill this <object> if it doesn't contain a polyfillable <embed> or
            // another <object> that would be supported on modern browsers.
            return (
                !Array.from(elem.getElementsByTagName("object")).some(
                    RuffleObject.isInterdictable,
                ) &&
                !Array.from(elem.getElementsByTagName("embed")).some(
                    RuffleEmbed.isInterdictable,
                )
            );
        } else if (classid) {
            // Non-Flash classid.
            return false;
        }

        return isSwf(filename, type);
    }

    /**
     * Creates a RuffleObject that will polyfill and replace the given element.
     *
     * @param elem Element to replace.
     * @returns Created RuffleObject.
     */
    static fromNativeObjectElement(elem: Element): RuffleObject {
        const externalName = registerElement("ruffle-object", RuffleObject);
        const ruffleObj: RuffleObject = document.createElement(
            externalName,
        ) as RuffleObject;

        // Avoid copying embeds-inside-objects to avoid double polyfilling.
        for (const embedElem of Array.from(
            elem.getElementsByTagName("embed"),
        )) {
            if (RuffleEmbed.isInterdictable(embedElem)) {
                embedElem.remove();
            }
        }

        // Avoid copying objects-inside-objects to avoid double polyfilling.
        // This may happen when Internet Explorer's conditional comments are used.
        for (const objectElem of Array.from(
            elem.getElementsByTagName("object"),
        )) {
            if (RuffleObject.isInterdictable(objectElem)) {
                objectElem.remove();
            }
        }

        ruffleObj.copyElement(elem);

        return ruffleObj;
    }
}
