import { copyElement, RufflePlayerElement } from "./ruffle-player-element";
import {
    getPolyfillOptions,
    isFallbackElement,
    isYoutubeFlashSource,
    workaroundYoutubeMixedContent,
} from "./inner";
import { registerElement } from "../register-element";
import { isSwf } from "../../swf-utils";

/**
 * A polyfill html element.
 *
 * This specific class tries to polyfill existing `<embed>` tags,
 * and should not be used. Prefer [[RufflePlayer]] instead.
 *
 * @internal
 */
export class RuffleEmbedElement extends RufflePlayerElement {
    /**
     * @ignore
     * @internal
     */
    override connectedCallback(): void {
        super.connectedCallback();
        const src = this.attributes.getNamedItem("src");
        if (src) {
            // Get the configuration options that have been overwritten for this movie.
            const getOptionString = (optionName: string) =>
                this.attributes.getNamedItem(optionName)?.value ?? null;
            const options = getPolyfillOptions(src.value, getOptionString);

            // Kick off the SWF download.
            this.load(options, true);
        }
    }

    /**
     * Polyfill of HTMLEmbedElement.
     *
     * @ignore
     * @internal
     */
    override get nodeName(): string {
        return "EMBED";
    }

    /**
     * Polyfill of HTMLEmbedElement.
     *
     * @ignore
     * @internal
     */
    get src(): string | undefined {
        return this.attributes.getNamedItem("src")?.value;
    }

    /**
     * Polyfill of HTMLEmbedElement.
     *
     * @ignore
     * @internal
     */
    set src(srcval: string | undefined) {
        if (srcval) {
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
    static override get observedAttributes(): string[] {
        return ["src", "width", "height"];
    }

    /**
     * @ignore
     * @internal
     */
    override attributeChangedCallback(
        name: string,
        oldValue: string | undefined,
        newValue: string | undefined,
    ): void {
        super.attributeChangedCallback(name, oldValue, newValue);
        if (this.isConnected && name === "src") {
            const src = this.attributes.getNamedItem("src");
            if (src) {
                const getOptionString = (optionName: string) =>
                    this.attributes.getNamedItem(optionName)?.value ?? null;
                const options = getPolyfillOptions(src.value, getOptionString);
                this.load(options, true);
            }
        }
    }

    /**
     * Checks if the given element may be polyfilled with this one.
     *
     * @param elem Element to check.
     * @returns True if the element looks like a Flash embed.
     */
    static isInterdictable(elem: Element): boolean {
        const src = elem.getAttribute("src");
        const type = elem.getAttribute("type");

        // Don't polyfill when no file is specified.
        if (!src) {
            return false;
        }

        // Don't polyfill if the element is inside a specific node.
        if (isFallbackElement(elem)) {
            return false;
        }

        // Don't polyfill when the file is a YouTube Flash source.
        if (isYoutubeFlashSource(src)) {
            // Workaround YouTube mixed content; this isn't what browsers do automatically, but while we're here, we may as well.
            workaroundYoutubeMixedContent(elem, "src");
            return false;
        }

        return isSwf(src, type);
    }

    /**
     * Creates a RuffleEmbed that will polyfill and replace the given element.
     *
     * @param elem Element to replace.
     * @returns Created RuffleEmbed.
     */
    static fromNativeEmbedElement(elem: Element): RuffleEmbedElement {
        const externalName = registerElement(
            "ruffle-embed",
            RuffleEmbedElement,
        );
        const ruffleObj = document.createElement(
            externalName,
        ) as RuffleEmbedElement;
        copyElement(elem, ruffleObj);

        return ruffleObj;
    }

    /**
     * Polyfill of height getter
     *
     * @ignore
     * @internal
     */
    get height(): string {
        return this.getAttribute("height") || "";
    }

    /**
     * Polyfill of height setter
     *
     * @ignore
     * @internal
     */
    set height(height: string) {
        this.setAttribute("height", height);
    }

    /**
     * Polyfill of width getter
     *
     * @ignore
     * @internal
     */
    get width(): string {
        return this.getAttribute("width") || "";
    }

    /**
     * Polyfill of width setter
     *
     * @ignore
     * @internal
     */
    set width(widthVal: string) {
        this.setAttribute("width", widthVal);
    }

    /**
     * Polyfill of type getter
     *
     * @ignore
     * @internal
     */
    get type(): string {
        return this.getAttribute("type") || "";
    }

    /**
     * Polyfill of type setter
     *
     * @ignore
     * @internal
     */
    set type(typeVal: string) {
        this.setAttribute("type", typeVal);
    }
}
