import { FLASH_MIMETYPE, FUTURESPLASH_MIMETYPE, is_swf_filename, RufflePlayer } from "./ruffle-player.js";
import { register_element } from "./register-element";

export default class RuffleEmbed extends RufflePlayer {
    constructor(...args) {
        let self = super(...args);

        return self;
    }

    connectedCallback() {
        super.connectedCallback();
        this.stream_swf_url(this.attributes.src.value);
    }

    get src() {
        return this.attributes.src.value;
    }

    set src(srcval) {
        this.attributes.src = srcval;
    }

    static get observedAttributes() {
        return ["src"];
    }

    attributeChangedCallback(name, oldValue, newValue) {
        super.attributeChangedCallback(name, oldValue, newValue);
        
        if (this.isConnected && name === "src") {
            this.stream_swf_url(this.attributes.src.value);
        }
    }

    static is_interdictable(elem) {
        if (elem.type === FLASH_MIMETYPE || elem.type === FUTURESPLASH_MIMETYPE) {
            return true;
        } else if (elem.type === undefined || elem.type === "") {
            return is_swf_filename(elem.src);
        }

        return false;
    }

    static from_native_embed_element(elem) {
        let external_name = register_element("ruffle-embed", RuffleEmbed);
        let ruffle_obj = document.createElement(external_name);
        for (let attrib of elem.attributes) {
            if (attrib.specified) {
                ruffle_obj.setAttribute(attrib.name, attrib.value);
            }
        }

        for (let node of Array.from(elem.children)) {
            ruffle_obj.appendChild(node);
        }

        return ruffle_obj;
    }
}