import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
    FLASH_ACTIVEX_CLASSID,
    is_swf_filename,
    RufflePlayer,
} from "./ruffle-player";
import { register_element } from "./register-element";

export class RuffleObject extends RufflePlayer {
    private params: Record<string, string> = {};

    constructor() {
        super();
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

        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }

        const parameters = RuffleObject.find_case_insensitive(
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
            this.streamSwfUrl(url, parameters);
        }
    }

    debugPlayerInfo() {
        let error_text = super.debugPlayerInfo();
        error_text += "Player type: Object\n";

        let url = null;

        if (this.attributes.getNamedItem("data")) {
            url = this.attributes.getNamedItem("data")?.value;
        } else if (this.params.movie) {
            url = this.params.movie;
        }
        error_text += `SWF URL: ${url}\n`;

        Object.keys(this.params).forEach((key) => {
            error_text += `Param ${key}: ${this.params[key]}\n`;
        });

        Object.keys(this.attributes).forEach((key) => {
            error_text += `Attribute ${key}: ${
                this.attributes.getNamedItem(key)?.value
            }\n`;
        });

        return error_text;
    }

    get data() {
        return this.attributes.getNamedItem("data")?.value;
    }

    set data(href) {
        if (href != undefined) {
            const attr = document.createAttribute("data");
            attr.value = href;
            this.attributes.setNamedItem(attr);
        } else {
            this.attributes.removeNamedItem("data");
        }
    }

    static is_interdictable(elem: HTMLElement) {
        const data = elem.attributes.getNamedItem("data")?.value.toLowerCase();
        if (!data) {
            let has_movie = false;
            const params = elem.getElementsByTagName("param");
            for (let i = 0; i < params.length; i++) {
                if (params[i].name == "movie" && params[i].value) {
                    has_movie = true;
                }
            }
            if (!has_movie) {
                return false;
            }
        }

        const type = elem.attributes.getNamedItem("type")?.value.toLowerCase();
        const classid = elem.attributes
            .getNamedItem("classid")
            ?.value.toLowerCase();
        if (
            type === FLASH_MIMETYPE.toLowerCase() ||
            type === FUTURESPLASH_MIMETYPE.toLowerCase() ||
            type === FLASH7_AND_8_MIMETYPE.toLowerCase() ||
            type === FLASH_MOVIE_MIMETYPE.toLowerCase()
        ) {
            return true;
        } else if (classid === FLASH_ACTIVEX_CLASSID.toLowerCase()) {
            return true;
        } else if (
            (type == null || type === "") &&
            (classid == null || classid === "")
        ) {
            const params = RuffleObject.params_of(elem);
            if (data && is_swf_filename(data)) {
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
    static find_case_insensitive(obj: any, key: string, defaultValue: any) {
        key = key.toLowerCase();
        for (const k in obj) {
            if (Object.hasOwnProperty.call(obj, k) && key === k.toLowerCase()) {
                return obj[k];
            }
        }
        return defaultValue;
    }

    static params_of(elem: HTMLElement) {
        const params: Record<string, string> = {};

        for (const param of elem.children) {
            if (param instanceof HTMLParamElement) {
                const key = param.attributes.getNamedItem("name")?.value;
                const value = param.attributes.getNamedItem("value")?.value;
                if (key && value) {
                    params[key] = value;
                }
            }
        }

        return params;
    }

    static from_native_object_element(elem: HTMLElement) {
        const external_name = register_element("ruffle-object", RuffleObject);
        const ruffle_obj: RuffleObject = <RuffleObject>(
            document.createElement(external_name)
        );
        ruffle_obj.copyElement(elem);

        return ruffle_obj;
    }
}
