import RufflePlayer from "./ruffle-player.js";

export default class RuffleEmbed extends RufflePlayer {
    constructor(...args) {
        let self = super(...args);

        return self;
    }

    connectedCallback() {
        super.connectedCallback();
        super.stream_swf_url(this.attributes.src.value);
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

        //TODO: We get a double play if we just naively load this twice.
        //Check if the element is connected before doing anything!
        if (name === "src") {
            //this.stream_swf_url(this.attributes.src.value);
        }
    }

    static is_interdictable(elem) {
        return elem.type === "application/x-shockwave-flash";
    }

    static from_native_embed_element(elem) {
        var ruffle_obj = document.createElement("ruffle-embed");
        for (let attrib of elem.attributes) {
            if (attrib.specified) {
                ruffle_obj.setAttribute(attrib.name, attrib.value);
            }
        }

        for (let node of elem.children) {
            ruffle_obj.appendChild(node);
        }

        return ruffle_obj;
    }
}