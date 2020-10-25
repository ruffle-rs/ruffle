const {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    FLASH_ACTIVEX_CLASSID,
    is_swf_filename,
    RufflePlayer,
} = require("./ruffle-player.js");
const { register_element } = require("./register-element");

module.exports = class RuffleObject extends RufflePlayer {
    constructor(...args) {
        super(...args);
    }

    connectedCallback() {
        super.connectedCallback();

        this.params = RuffleObject.params_of(this);

        const allowScriptAccess = RuffleObject.find_case_insensitive(
            this.params,
            "allowScriptAccess",
            "sameDomain"
        );
        let url = null;

        if (this.attributes.data) {
            url = this.attributes.data.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }

        let parameters = RuffleObject.find_case_insensitive(
            this.params,
            "flashvars",
            RuffleObject.find_case_insensitive(
                this.attributes,
                "flashvars",
                null
            )
        );

        if (url) {
            this.allow_script_access =
                allowScriptAccess &&
                (allowScriptAccess.toLowerCase() === "always" ||
                    (allowScriptAccess.toLowerCase() === "samedomain" &&
                        new URL(window.location.href).origin ===
                            new URL(url, window.location.href).origin));

            //Kick off the SWF download.
            this.stream_swf_url(url, parameters);
        }
    }

    debug_player_info() {
        let error_text = super.debug_player_info();
        error_text += "Player type: Object\n";

        let url = null;

        if (this.attributes.data) {
            url = this.attributes.data.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }
        error_text += `SWF URL: ${url}\n`;

        Object.keys(this.params).forEach((key) => {
            error_text += `Param ${key}: ${this.params[key]}\n`;
        });

        Object.keys(this.attributes).forEach((key) => {
            error_text += `Attribute ${key}: ${this.attributes[key]}\n`;
        });

        return error_text;
    }

    get data() {
        return this.attributes.data.value;
    }

    set data(href) {
        this.attributes.data = href;
    }

    static is_interdictable(elem) {
        if (!elem.data) {
            let has_movie = false;
            let params = elem.getElementsByTagName("param");
            for (let i = 0; i < params.length; i++) {
                if (params[i].name == "movie" && params[i].value) {
                    has_movie = true;
                }
            }
            if (!has_movie) {
                return false;
            }
        }
        if (
            elem.type.toLowerCase() === FLASH_MIMETYPE.toLowerCase() ||
            elem.type.toLowerCase() === FUTURESPLASH_MIMETYPE.toLowerCase() ||
            elem.type.toLowerCase() == FLASH7_AND_8_MIMETYPE.toLowerCase() ||
            elem.type.toLowerCase() == FLASH_MOVIE_MIMETYPE.toLowerCase()
        ) {
            return true;
        } else if (
            elem.attributes &&
            elem.attributes.classid &&
            elem.attributes.classid.value.toLowerCase() ===
                FLASH_ACTIVEX_CLASSID.toLowerCase()
        ) {
            return true;
        } else if (
            (elem.type === undefined || elem.type === "") &&
            elem.attributes.classid === undefined
        ) {
            let params = RuffleObject.params_of(elem);
            if (elem.data && is_swf_filename(elem.data)) {
                return true;
            } else if (
                params &&
                params.movie &&
                is_swf_filename(params.movie)
            ) {
                return true;
            }
        }

        return false;
    }

    /*
     * Find and return the first value in obj with the given key.
     * Many Flash params were case insensitive, so we use this when checking for them.
     */
    static find_case_insensitive(obj, key, defaultValue) {
        key = key.toLowerCase();
        for (const k in obj) {
            if (Object.hasOwnProperty.call(obj, k) && key === k.toLowerCase()) {
                return obj[k];
            }
        }
        return defaultValue;
    }

    static params_of(elem) {
        let params = {};

        for (let param of elem.children) {
            if (param.constructor === HTMLParamElement) {
                params[param.name] = param.value;
            }
        }

        return params;
    }

    static from_native_object_element(elem) {
        let external_name = register_element("ruffle-object", RuffleObject);
        let ruffle_obj = document.createElement(external_name);
        ruffle_obj.copy_element(elem);

        return ruffle_obj;
    }
};
