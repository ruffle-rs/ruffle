import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    is_swf_filename,
    RufflePlayer,
} from "./ruffle-player";
import { register_element } from "./register-element";

export class RuffleEmbed extends RufflePlayer {
    constructor() {
        super();
    }

    connectedCallback() {
        super.connectedCallback();
        let parameters = null;
        const flashvars = this.attributes.getNamedItem("flashvars");
        if (flashvars) {
            parameters = flashvars.value;
        }
        const src = this.attributes.getNamedItem("src");
        if (src) {
            this.streamSwfUrl(src.value, parameters);
        }
    }

    get src() {
        return this.attributes.getNamedItem("src")?.value;
    }

    set src(srcval) {
        if (srcval != undefined) {
            const attr = document.createAttribute("src");
            attr.value = srcval;
            this.attributes.setNamedItem(attr);
        } else {
            this.attributes.removeNamedItem("src");
        }
    }

    static get observedAttributes() {
        return ["src", "width", "height"];
    }

    attributeChangedCallback(
        name: string,
        oldValue: string | undefined,
        newValue: string | undefined
    ) {
        super.attributeChangedCallback(name, oldValue, newValue);
        if (this.isConnected && name === "src") {
            let parameters = null;
            const flashvars = this.attributes.getNamedItem("flashvars");
            if (flashvars) {
                parameters = flashvars.value;
            }
            const src = this.attributes.getNamedItem("src");
            if (src) {
                this.streamSwfUrl(src.value, parameters);
            }
        }
    }

    static is_interdictable(elem: HTMLElement) {
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
            return is_swf_filename(elem.getAttribute("src"));
        }

        return false;
    }

    static from_native_embed_element(elem: HTMLElement) {
        const external_name = register_element("ruffle-embed", RuffleEmbed);
        const ruffle_obj = <RuffleEmbed>document.createElement(external_name);
        ruffle_obj.copy_element(elem);

        return ruffle_obj;
    }
}
