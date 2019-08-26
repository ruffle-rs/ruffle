import {FLASH_MIMETYPE, FUTURESPLASH_MIMETYPE, FLASH_ACTIVEX_CLASSID, RufflePlayer} from "./ruffle-player.js";

export default class RuffleObject extends RufflePlayer {
    constructor(...args) {
        return super(...args);
    }

    connectedCallback() {
        super.connectedCallback();
        
        this.params = RuffleObject.params_of(this);
        
        //Kick off the SWF download.
        if (this.params.movie) {
            super.stream_swf_url(this.params.movie);
        }
    }

    static is_interdictable(elem) {
        return elem.type === FLASH_MIMETYPE || elem.type === FUTURESPLASH_MIMETYPE || elem.attributes.classid.value === FLASH_ACTIVEX_CLASSID;
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
        var ruffle_obj = document.createElement("ruffle-object");
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