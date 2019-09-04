import {FLASH_MIMETYPE, FUTURESPLASH_MIMETYPE, RufflePlayer} from "./ruffle-player.js";

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
        return self.attributes.src;
    }

    set src(srcval) {
        self.attributes.src = srcval;
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
        return elem.type === FLASH_MIMETYPE || elem.type === FUTURESPLASH_MIMETYPE;
    }

    static from_native_embed_element(elem) {
        var ruffle_obj = document.createElement("ruffle-embed");
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