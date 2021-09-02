import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    isBuiltInContextMenuVisible,
    isScriptAccessAllowed,
    isSwfFilename,
    RufflePlayer,
} from "./ruffle-player";
import { registerElement } from "./register-element";

/**
 * A polyfill html element.
 *
 * This specific class tries to polyfill existing `<embed>` tags,
 * and should not be used. Prefer [[RufflePlayer]] instead.
 *
 * @internal
 */
export class RuffleEmbed extends RufflePlayer {
    /**
     * Constructs a new Ruffle flash player for insertion onto the page.
     *
     * This specific class tries to polyfill existing `<embed>` tags,
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
        const src = this.attributes.getNamedItem("src");
        if (src) {
            const allowScriptAccess =
                this.attributes.getNamedItem("allowScriptAccess")?.value ??
                null;
            const menu = this.attributes.getNamedItem("menu")?.value ?? null;

            // Kick off the SWF download.
            this.load({
                url: src.value,
                allowScriptAccess: isScriptAccessAllowed(
                    allowScriptAccess,
                    src.value
                ),
                parameters: this.attributes.getNamedItem("flashvars")?.value,
                backgroundColor: this.attributes.getNamedItem("bgcolor")?.value,
                base: this.attributes.getNamedItem("base")?.value,
                menu: isBuiltInContextMenuVisible(menu),
                salign: this.attributes.getNamedItem("salign")?.value ?? "",
                quality:
                    this.attributes.getNamedItem("quality")?.value ?? "high",
                scale:
                    this.attributes.getNamedItem("scale")?.value ?? "showAll",
            });
        }
    }

    /**
     * Polyfill of HTMLObjectElement.
     *
     * @ignore
     * @internal
     */
    get src(): string | undefined {
        return this.attributes.getNamedItem("src")?.value;
    }

    /**
     * Polyfill of HTMLObjectElement.
     *
     * @ignore
     * @internal
     */
    set src(srcval: string | undefined) {
        if (srcval != undefined) {
            const attr = document.createAttribute("src");
            attr.value = srcval;
            this.attributes.setNamedItem(attr);
        } else {
            this.attributes.removeNamedItem("src");
        }
    }

    /**
     * @ignore
     * @internal
     */
    static get observedAttributes(): string[] {
        return ["src", "width", "height"];
    }

    /**
     * @ignore
     * @internal
     */
    attributeChangedCallback(
        name: string,
        oldValue: string | undefined,
        newValue: string | undefined
    ): void {
        super.attributeChangedCallback(name, oldValue, newValue);
        if (this.isConnected && name === "src") {
            let parameters;
            const flashvars = this.attributes.getNamedItem("flashvars");
            if (flashvars) {
                parameters = flashvars.value;
            }
            const src = this.attributes.getNamedItem("src");
            if (src) {
                this.load({
                    url: src.value,
                    parameters,
                    base: this.attributes.getNamedItem("base")?.value,
                });
            }
        }
    }

    /**
     * Checks if the given element may be polyfilled with this one.
     *
     * @param elem Element to check.
     * @returns True if the element looks like a flash embed.
     */
    static isInterdictable(elem: HTMLElement): boolean {
        if (!elem.getAttribute("src")) {
            return false;
        }
        const type = elem.getAttribute("type")?.toLowerCase();
        if (
            type === FLASH_MIMETYPE.toLowerCase() ||
            type === FUTURESPLASH_MIMETYPE.toLowerCase() ||
            type === FLASH7_AND_8_MIMETYPE.toLowerCase() ||
            type === FLASH_MOVIE_MIMETYPE.toLowerCase()
        ) {
            return true;
        } else if (type == null || type === "") {
            return isSwfFilename(elem.getAttribute("src"));
        }

        return false;
    }

    /**
     * Creates a RuffleEmbed that will polyfill and replace the given element.
     *
     * @param elem Element to replace.
     * @returns Created RuffleEmbed.
     */
    static fromNativeEmbedElement(elem: HTMLElement): RuffleEmbed {
        const externalName = registerElement("ruffle-embed", RuffleEmbed);
        const ruffleObj = <RuffleEmbed>document.createElement(externalName);
        ruffleObj.copyElement(elem);

        return ruffleObj;
    }
}
