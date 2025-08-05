// This must be in global scope because `document.currentScript`
// works only while the script is initially being processed.
export let currentScriptURL: URL | null = null;
export let isExtension = false;

try {
    if (
        document.currentScript instanceof HTMLScriptElement &&
        document.currentScript.src !== ""
    ) {
        let src = document.currentScript.src;

        // CDNs allow omitting the filename. If it's omitted, append a slash to
        // prevent the last component from being dropped.
        if (!src.endsWith(".js") && !src.endsWith("/")) {
            src += "/";
        }

        currentScriptURL = new URL(".", src);
        isExtension = currentScriptURL.protocol.includes("extension");
    }
} catch (e) {
    console.warn("Unable to get currentScript URL", e);
}

/**
 * Sets the current script URL and isExtension boolean manually when using the extension.
 *
 * @param src The location of Ruffle's resources within the extension.
 */
export function setCurrentScriptURL(src: URL) {
    currentScriptURL = src;
    isExtension = currentScriptURL.protocol.includes("extension");
}
