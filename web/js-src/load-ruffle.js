/**
 * Conditional ruffle loader
 */

/**
 * Load ruffle from an automatically-detected location.
 * 
 * This function returns a new instance of Ruffle and downloads it every time.
 * You should not use it directly; this module will memoize the resource
 * download.
 */
async function fetch_ruffle() {
    if (!window.runtime_path) {
        //Executing on the demo site (which is fully webpack) so we can just use
        //webpack to get the Ruffle chunk.
        let ruffle_module = await import("../pkg/ruffle");
        return ruffle_module.Ruffle;
    }
    
    //Download Ruffle from the URL the browser gives us.
    let ruffle_bindings_url = window.runtime_path + "dist/ruffle_web.js";
    let ruffle_wasm_url = window.runtime_path + "dist/ruffle_web_bg.wasm";

    //We load the wasm package early so that both requests are parallelized.
    //This won't be awaited by us at all.
    let ruffle_wasm_request = fetch(ruffle_wasm_url);

    //One point of admin: `wasm-pack`, in no-modules mode, stores it's bindings
    //straight in `window`, which we don't want. We grab whatever was in Window
    //before loading in the module so we can replace what was there.
    let ruffle_bindings_request = await fetch(ruffle_bindings_url);
    let ruffle_bindings_src = await ruffle_bindings_request.text();
    let noconflict_wasm_bindgen = window.wasm_bindgen;
    (new Function(ruffle_bindings_src))();
    let ruffle_wasm_bindgen = window.wasm_bindgen;
    window.wasm_bindgen = noconflict_wasm_bindgen;

    //Next step: Actually initialize our bindings.
    let ruffle_wasm_response = await ruffle_wasm_request;
    let ruffle_wasm_data = await ruffle_wasm_response.arrayBuffer();
    let ruffle_bindings = await ruffle_wasm_bindgen(ruffle_wasm_data).catch(function (e) {
        console.error(e);
    });

    return ruffle_wasm_bindgen.Ruffle;
}

let last_loaded_ruffle = null;

/**
 * Obtain an instance of `Ruffle`.
 * 
 * This function returns a promise which yields `Ruffle` asynchronously.
 */
export default function load_ruffle() {
    if (last_loaded_ruffle == null) {
        last_loaded_ruffle = fetch_ruffle();
    }

    return last_loaded_ruffle;
}