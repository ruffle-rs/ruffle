import {FLASH_MIMETYPE, FUTURESPLASH_MIMETYPE, FLASH_ACTIVEX_CLASSID} from "./ruffle-player.js";

/**
 * Detect Flash using a handful of APIs.
 * 
 * If you run this function after loading `plugin-polyfill`, you will only see
 * the info we added to the current window, which is wrong. Alternatively, if
 * you want to error-check your plugin falsification, use this function after
 * installing the falsification.
 * 
 * This detect is largely similar to the one used by `swfobject` but with
 * certain browser detects removed.
 * 
 * Returns if the browser has Flash and the version we detected.
 */
export default function detect_flash() {
    let has_flash = false;
    let version = [0,0,0];

    if (navigator && navigator.plugins &&
        navigator.plugins["Shockwave Flash"] &&
        navigator.plugins["Shockwave Flash"].description) {
        
        let description = navigator.plugins["Shockwave Flash"].description;

        if (navigator.mimeTypes &&
            navigator.mimeTypes[FLASH_MIMETYPE] &&
            navigator.mimeTypes[FLASH_MIMETYPE].enabledPlugin) {
            
            has_flash = true;
            description = description.replace(/^.*\s+(\S+\s+\S+$)/, "$1");
            version[0] = toInt(description.replace(/^(.*)\..*$/, "$1"));
            version[1] = toInt(description.replace(/^.*\.(.*)\s.*$/, "$1"));
            version[2] = /[a-zA-Z]/.test(description) ? toInt(description.replace(/^.*[a-zA-Z]+(.*)$/, "$1")) : 0;
        }
    } else if (window && window.ActiveXObject) {
        try {
            let activex_flash = new window.ActiveXObject("ShockwaveFlash.ShockwaveFlash");
            if (activex_flash) {
                let player_version = activex_flash.GetVariable("$version");

                if (player_version) {
                    version = player_version.split(" ")[1].split(",").map(toInt);
                }
            }
        } catch (e) {
        }
    }

    return (has_flash, version);
}