const {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    is_swf_filename,
    RufflePlayer,
} = require("./ruffle-player.js");
const { register_element } = require("./register-element");

module.exports = class RuffleEmbed extends RufflePlayer {
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
        return ["src", "width", "height"];
    }

    attributeChangedCallback(name, oldValue, newValue) {
        super.attributeChangedCallback(name, oldValue, newValue);
        console.log(name + " " + oldValue + " " + newValue);
        if (this.isConnected && name === "src") {
            this.stream_swf_url(this.attributes.src.value);
        }
    }

    static is_interdictable(elem) {
        if (!elem.src) {
            return false;
        }
        if (
            elem.type.toLowerCase() === FLASH_MIMETYPE.toLowerCase() ||
            elem.type.toLowerCase() === FUTURESPLASH_MIMETYPE.toLowerCase() ||
            elem.type.toLowerCase() == FLASH7_AND_8_MIMETYPE.toLowerCase() ||
            elem.type.toLowerCase() == FLASH_MOVIE_MIMETYPE.toLowerCase()
        ) {
            return true;
        } else if (elem.type === undefined || elem.type === "") {
            return is_swf_filename(elem.src);
        }

        return false;
    }

    static from_native_embed_element(elem) {
        let external_name = register_element("ruffle-embed", RuffleEmbed);
        let ruffle_obj = document.createElement(external_name);
        ruffle_obj.copy_element(elem);

        return ruffle_obj;
    }
};
