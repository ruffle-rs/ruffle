import { FLASH_MIMETYPE, FUTURESPLASH_MIMETYPE, FLASH7_AND_8_MIMETYPE, FLASH_MOVIE_MIMETYPE, FLASH_ACTIVEX_CLASSID, is_swf_filename, RufflePlayer } from "./ruffle-player.js";
import { register_element } from "./register-element";

export default class RuffleObject extends RufflePlayer {
    constructor(...args) {
        super(...args);
    }

    connectedCallback() {
        super.connectedCallback();
        
        this.params = RuffleObject.params_of(this);

        //Kick off the SWF download.
        if (this.attributes.data) {
            this.stream_swf_url(this.attributes.data.value);
        } else if (this.params.movie) {
            this.stream_swf_url(this.params.movie);
        }
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
            for (let i = 0;i < params.length;i ++) {
                if (params[i].name == "movie" && params[i].value) {
                    has_movie = true;
                }
            }
            if (!has_movie) {
                return false;
            }
        }
        if (elem.type === FLASH_MIMETYPE || elem.type === FUTURESPLASH_MIMETYPE || elem.type == FLASH7_AND_8_MIMETYPE || elem.type == FLASH_MOVIE_MIMETYPE) {
            return true;
        } else if (elem.attributes && elem.attributes.classid && elem.attributes.classid.value === FLASH_ACTIVEX_CLASSID) {
            return true;
        } else if ((elem.type === undefined || elem.type === "") && elem.attributes.classid === undefined) {
            let params = RuffleObject.params_of(elem);
            if (params && params.movie) {
                return is_swf_filename(params.movie);
            }
        }

        return false;
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
}
