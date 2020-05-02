/**
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
let page_optout = document.getElementsByTagName("html")[0].dataset.ruffleOptout !== undefined;
let obfuscated_event_prefix = "rufEvent" + Math.floor(Math.random() * 100000000000);
let ext_path = "";
if (chrome && chrome.extension && chrome.extension.getURL) {
    ext_path = chrome.extension.getURL("dist/ruffle.js").replace("dist/ruffle.js", "");
} else if (browser && browser.runtime && browser.runtime.getURL) {
    ext_path = browser.runtime.getURL("dist/ruffle.js").replace("dist/ruffle.js", "");
}
if (!(page_optout||window.RufflePlayer)) {
    let setup_scriptelem = document.createElement("script");
    let setup_src = "var runtime_path = \"" +
        ext_path + "\";\nvar obfuscated_event_prefix = \"" +
        obfuscated_event_prefix + "\";" +
    '(function(){class RuffleMimeType{constructor(a,b,c){this.type=a,this.description=b,this.suffixes=c}}class RuffleMimeTypeArray{constructor(a){this.__mimetypes=[],this.__named_mimetypes={};for(let b of a)this.install(b)}install(a){let b=this.__mimetypes.length;this.__mimetypes.push(a),this.__named_mimetypes[a.type]=a,this[a.type]=a,this[b]=a}item(a){return this.__mimetypes[a]}namedItem(a){return this.__named_mimetypes[a]}get length(){return this.__mimetypes.length}}class RufflePlugin extends RuffleMimeTypeArray{constructor(a,b,c,d){super(d),this.name=a,this.description=b,this.filename=c}install(a){a.enabledPlugin||(a.enabledPlugin=this),super.install(a)}}class RufflePluginArray{constructor(a){this.__plugins=[],this.__named_plugins={};for(let b of a)this.install(b)}install(a){let b=this.__plugins.length;this.__plugins.push(a),this.__named_plugins[a.name]=a,this[a.name]=a,this[b]=a}item(a){return this.__plugins[a]}namedItem(a){return this.__named_plugins[a]}get length(){return this.__plugins.length}}const FLASH_PLUGIN=new RufflePlugin("Shockwave Flash","Shockwave Flash 32.0 r0","ruffle.js",[new RuffleMimeType("application/futuresplash","Shockwave Flash","spl"),new RuffleMimeType("application/x-shockwave-flash","Shockwave Flash","swf"),new RuffleMimeType("application/x-shockwave-flash2-preview","Shockwave Flash","swf"),new RuffleMimeType("application/vnd.adobe.flash-movie","Shockwave Flash","swf")]);function install_plugin(a){console.log("installing polyfill");navigator.plugins.install||Object.defineProperty(navigator,"plugins",{value:new RufflePluginArray(navigator.plugins),writable:!1}),navigator.plugins.install(a),0<a.length&&!navigator.mimeTypes.install&&Object.defineProperty(navigator,"mimeTypes",{value:new RuffleMimeTypeArray(navigator.mimeTypes),writable:!1});for(var b=0;b<a.length;b+=1)navigator.mimeTypes.install(a[b])}install_plugin(FLASH_PLUGIN);})();';
    let scriptelem = document.createElement("script");
    setup_scriptelem.innerHTML = setup_src;
    (document.head || document.documentElement).appendChild(setup_scriptelem);
    window.RufflePlayer = {};
    window.RufflePlayer.config = {
        "public_path": ext_path + "dist/",
        "polyfills": ["static-content", "dynamic-content"]
    };
    scriptelem.src = ext_path + "dist/ruffle.js";
    (document.head || document.documentElement).appendChild(scriptelem);
}
