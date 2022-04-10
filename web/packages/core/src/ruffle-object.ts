import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    FLASH_ACTIVEX_CLASSID,
    isBuiltInContextMenuVisible,
    isFallbackElement,
    isScriptAccessAllowed,
    isSwfFilename,
    isYoutubeFlashSource,
    workaroundYoutubeMixedContent,
    RufflePlayer,
} from "./ruffle-player";
import { registerElement } from "./register-element";
import { URLLoadOptions } from "./load-options";
import { RuffleEmbed } from "./ruffle-embed";

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

        let url = null;
        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }

        const allowScriptAccess = findCaseInsensitive(
            this.params,
            "allowScriptAccess",
            null
        );

        const parameters = findCaseInsensitive(
            this.params,
            "flashvars",
            this.getAttribute("flashvars")
        );

        const backgroundColor = findCaseInsensitive(
            this.params,
            "bgcolor",
            this.getAttribute("bgcolor")
        );

        const base = findCaseInsensitive(
            this.params,
            "base",
            this.getAttribute("base")
        );

        const menu = findCaseInsensitive(this.params, "menu", null);
        const salign = findCaseInsensitive(this.params, "salign", "");
        const quality = findCaseInsensitive(this.params, "quality", "high");
        const scale = findCaseInsensitive(this.params, "scale", "showAll");

        if (url) {
            const options: URLLoadOptions = { url };
            options.allowScriptAccess = isScriptAccessAllowed(
                allowScriptAccess,
                url
            );
            if (parameters) {
                options.parameters = parameters;
            }
            if (backgroundColor) {
                options.backgroundColor = backgroundColor;
            }
            if (base) {
                options.base = base;
            }
            options.menu = isBuiltInContextMenuVisible(menu);
            if (salign) {
                options.salign = salign;
            }
            if (quality) {
                options.quality = quality;
            }
            if (scale) {
                options.scale = scale;
            }

            // Kick off the SWF download.
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

        // Don't polyfill if no movie specified.
        const data = elem.attributes.getNamedItem("data")?.value.toLowerCase();
        const params = paramsOf(elem);
        let isSwf;
        // Check for SWF file.
        if (data) {
            // Don't polyfill when the file is a Youtube Flash source.
            if (isYoutubeFlashSource(data)) {
                // Workaround YouTube mixed content; this isn't what browsers do automatically, but while we're here, we may as well
                workaroundYoutubeMixedContent(elem, "data");
                return false;
            }
            isSwf = isSwfFilename(data);
        } else if (params && params.movie) {
            // Don't polyfill when the file is a Youtube Flash source.
            if (isYoutubeFlashSource(params.movie)) {
                // Workaround YouTube mixed content; this isn't what browsers do automatically, but while we're here, we may as well
                const movie_elem = elem.querySelector(
                    "param[name='movie']"
                ) as HTMLElement;
                if (movie_elem) {
                    workaroundYoutubeMixedContent(movie_elem, "value");
                    // The data attribute needs to be set for the re-fetch to happen
                    // It also needs to be set on Firefox for the YouTube object rewrite to work, regardless of mixed content
                    const movie_src = movie_elem.getAttribute("value");
                    if (movie_src) {
                        elem.setAttribute("data", movie_src);
                    }
                }
                return false;
            }
            isSwf = isSwfFilename(params.movie);
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
                    RuffleObject.isInterdictable
                ) &&
                !Array.from(elem.getElementsByTagName("embed")).some(
                    RuffleEmbed.isInterdictable
                )
            );
        } else if (classid != null && classid !== "") {
            // Non-Flash classid.
            return false;
        }

        // Check for MIME type.
        const type = elem.attributes.getNamedItem("type")?.value.toLowerCase();
        if (
            type === FLASH_MIMETYPE.toLowerCase() ||
            type === FUTURESPLASH_MIMETYPE.toLowerCase() ||
            type === FLASH7_AND_8_MIMETYPE.toLowerCase() ||
            type === FLASH_MOVIE_MIMETYPE.toLowerCase()
        ) {
            return true;
        } else if (type != null && type !== "") {
            return false;
        }

        // If no MIME/class type is specified, polyfill if movie is an SWF file.
        return isSwf;
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

        // Avoid copying embeds-inside-objects to avoid double polyfilling.
        for (const embedElem of Array.from(
            elem.getElementsByTagName("embed")
        )) {
            if (RuffleEmbed.isInterdictable(embedElem)) {
                embedElem.remove();
            }
        }

        // Avoid copying objects-inside-objects to avoid double polyfilling.
        // This may happen when Internet Explorer's conditional comments are used.
        for (const objectElem of Array.from(
            elem.getElementsByTagName("object")
        )) {
            if (RuffleObject.isInterdictable(objectElem)) {
                objectElem.remove();
            }
        }

        ruffleObj.copyElement(elem);

        return ruffleObj;
    }
}
