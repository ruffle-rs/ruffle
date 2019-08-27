import detect_flash from "../../js-src/detect-native-flash";

function bind_boolean_setting(checkbox_elem) {
    let name = checkbox_elem.name,
        default_val = checkbox_elem.checked,
        get_obj = {};
    
    get_obj[name] = default_val;

    chrome.storage.sync.get(get_obj, function (items) {
        ruffle_enable.checked = items.ruffle_enable === true;
    });

    chrome.storage.onChanged.addListener(function (changes, namespace) {
        if (changes.hasOwnProperty(name)) {
            checkbox_elem.checked = changes[name] === true;
        }
    });

    ruffle_enable.addEventListener("click", function (e) {
        chrome.storage.sync.set(name, ruffle_enable.checked);
    });
}

document.addEventListener("DOMContentLoaded", function (e) {
    bind_boolean_setting(document.getElementById("ruffle_enable"));

    //Flash detect (not working yet)
    let current_flash_version = document.getElementById("current_flash_version");
    if (current_flash_version === null) {
        debugger;
    }
    let detect_result = detect_flash();
    let has_flash = detect_result[0];
    let version = detect_result[1];

    if (!has_flash) {
        current_flash_version.textContent = "No native Flash installed."
    } else {
        current_flash_version.textContent = "Flash Plugin detected, version " + version.join(".");
    }
});