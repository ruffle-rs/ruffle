(/**
 * Pierce the extension sandbox by copying our code into window space.
 * 
 * The isolation extension content scripts get is neat, but it causes problems
 * based on what browser you use:
 * 
 * 1. On Chrome, you are explicitly banned from registering custom elements
 * 2. On Firefox, you can register custom elements but they can't expose any
 *    useful API surface, and can't even see their own methods.
 * 
 * This code exists to pierce the extension sandbox, while maintaining:
 * 
 * 1. The isolation of not interfering with the page's execution environment
 *    unintentionally.
 * 2. The ability to load extension resources such as .wasm files
 */
async function() {
    let ext_path = browser.runtime.getURL("dist/ruffle.js").replace("dist/ruffle.js", "");
    let ruffle_src_resp = await fetch(ext_path + "dist/ruffle.js");
    if (ruffle_src_resp.ok) {
        let ruffle_src = "(function () { var runtime_path = \"" + ext_path + "\";\n" + await ruffle_src_resp.text() + "}())";
        let scriptelem = document.createElement("script");
        scriptelem.appendChild(document.createTextNode(ruffle_src));
        document.head.appendChild(scriptelem);
    } else {
        console.error("Critical error loading Ruffle into page")
    }
}());