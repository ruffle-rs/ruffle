import { FLASH_MIMETYPE, FUTURESPLASH_MIMETYPE, FLASH_ACTIVEX_CLASSID, is_swf_filename, RufflePlayer } from "./ruffle-player.js";
import { register_element } from "./register-element";

export default class RuffleObject extends RufflePlayer {
    constructor(...args) {
        return super(...args);
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
        if (elem.type === FLASH_MIMETYPE || elem.type === FUTURESPLASH_MIMETYPE) {
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