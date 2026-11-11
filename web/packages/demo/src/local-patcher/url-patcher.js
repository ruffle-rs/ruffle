import { decode, encode } from "base85";
import { Buffer } from "buffer";
import { gzipSync, gunzipSync } from "zlibjs";

// pre-define RUFFLE_RES
// const RUFFLE_RES: Record<string, [length: number, encoded: string, type: string]>;

// decode base85-gzip encoded file
function deb85gz(data) {
    let len = data[0];
    let decoded = len ? decode(data[1], "z85") : Buffer.alloc(0);
    // the file size for z85 needs to be a multiple of 4
    if (len != decoded.length) {
        let truncated = Buffer.alloc(len);
        decoded.copy(truncated, 0, 0, len);
        decoded = truncated;
    }
    decoded = gunzipSync(decoded, { noBuffer: true, H: true }); // noBuffer is mangled to H in zlibjs
    return URL.createObjectURL(new Blob([decoded], { type: data[2] }));
}

// encode base85-gzip file
function enb85gz(data) {
    let zipped = Buffer.from(gzipSync(data, { noBuffer: true, H: true })); // noBuffer is mangled to H in zlibjs
    let fileLength = zipped.length;
    // the file size for z85 needs to be a multiple of 4
    if (fileLength % 4 != 0) {
        let newFileContent = Buffer.alloc(Math.ceil(fileLength / 4) * 4);
        zipped.copy(newFileContent);
        zipped = newFileContent;
    }
    let encoded = encode(zipped, "z85");
    return [fileLength, encoded];
}

const decodedRes = {};

function decodeResourceIfExist(name) {
    if (typeof name != "string") {
        // skip undefined
        return null;
    }
    if (name.startsWith("blob:")) {
        // skip blob: urls
        return name;
    }
    const slashIndex = name.indexOf("/");
    if (slashIndex >= 0) {
        // leave filename only
        name = name.substring(slashIndex + 1);
    }
    if (decodedRes[name]) {
        // cached resource
        return decodedRes[name];
    }
    if (!RUFFLE_RES[name]) {
        // resource does not exist
        console.warn("url-patcher: unable to patch url", name);
        return null;
    }
    console.log("url-patcher: patching url", name);
    const decoded = deb85gz(RUFFLE_RES[name]);
    decodedRes[name] = decoded;
    RUFFLE_RES[name].length = 0; // reduce memory usage
    return decoded;
}

// patch url
const OriginalURL = URL;
function PatchedURL() {
    if (!new.target) throw new TypeError("'new' is required");
    const decodedURL = decodeResourceIfExist(arguments[0]);
    let self;
    if (decodedURL == null) {
        // URL cannot be decoded
        self = Reflect.construct(OriginalURL, arguments, new.target);
    } else {
        // Use decoded URL
        self = Reflect.construct(OriginalURL, [decodedURL], new.target);
    }
    return self;
}
PatchedURL.prototype = Object.create(OriginalURL.prototype, {
    constructor: {
        value: PatchedURL,
        writable: true,
        configurable: true,
    },
});
Object.setPrototypeOf(PatchedURL, OriginalURL);
Object.defineProperty(window, "URL", {
    value: PatchedURL,
    writable: true,
    configurable: true,
    enumerable: false,
});

// patch fetch
const originalFetch = fetch;
function patchedFetch() {
    const decodedURL = decodeResourceIfExist(arguments[0]);
    if (decodedURL) {
        return originalFetch(decodedURL);
    } else {
        return originalFetch.apply(this, arguments);
    }
}
Object.defineProperty(window, "fetch", {
    value: patchedFetch,
    writable: true,
    configurable: true,
    enumerable: false,
});

// RuffleBypassFileOriginCheck must be pre-defined and it is used in panic.tsx to bypass file: origin check
window.RuffleBypassFileOriginCheck = true;
// RufflePlayerDynSrc must be pre-defined and it is used in inline-html to process imports
window.RufflePlayerDynSrc = decodeResourceIfExist;
