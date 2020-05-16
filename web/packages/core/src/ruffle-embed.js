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
        const observer = new MutationObserver(function () {
            /* handle if original object is (re)moved */
            RufflePlayer.handle_player_changes(
                document.getElementsByTagName("ruffle-embed")
            );
        });
        observer.observe(document, { childList: true, subtree: true });
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
        if (elem.hasAttribute("data-polyfilled")) {
            return false;
        }
        if (!elem.src) {
            return false;
        }
        if (
            elem.parentElement &&
            elem.parentElement.tagName.toLowerCase() == "object" &&
            !elem.parentElement.hasAttribute("data-broken")
        ) {
            /* Only polyfill top-level objects */
            if (elem.hasAttribute("src")) {
                elem.removeAttribute("src");
            }
            elem.style.setProperty("display", "none", "important");
            /* Turn element into dummy */
            /* setting it to 0 width & height prevents it from *
             * messing up display Netscape 4 style             */
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
        const observer = new MutationObserver(
            RufflePlayer.handleOriginalAttributeChanges
        );
        ruffle_obj.copy_element(elem);
        ruffle_obj.original = elem;
        /* Set original for detecting if original is (re)moved */
        if (elem.hasAttribute("src")) {
            elem.removeAttribute("src");
        }
        if (elem.hasAttribute("id")) {
            elem.removeAttribute("id");
        }
        if (elem.hasAttribute("name")) {
            elem.removeAttribute("name");
        }
        elem.setAttribute("data-polyfilled", "polyfilled");
        elem.style.setProperty("display", "none", "important");
        /* Turn the original embed into a dummy element */
        observer.observe(elem, { attributes: true });

        return ruffle_obj;
    }
};
