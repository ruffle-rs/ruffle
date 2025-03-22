// This must be in global scope because `document.currentScript`
// works only while the script is initially being processed.
export let currentScriptURL: URL | null = null;
export let isExtension = false;

try {
    if (
        document.currentScript !== undefined &&
        document.currentScript !== null &&
        "src" in document.currentScript &&
        document.currentScript.src !== ""
    ) {
        let src = getScriptOriginalSrc(document.currentScript);

        // CDNs allow omitting the filename. If it's omitted, append a slash to
        // prevent the last component from being dropped.
        if (!src.endsWith(".js") && !src.endsWith("/")) {
            src += "/";
        }

        currentScriptURL = new URL(".", src);
        isExtension = currentScriptURL.protocol.includes("extension");
    }
} catch (_e) {
    console.warn("Unable to get currentScript URL");
}

/**
 *
 * Obtain the origin src content according to the running environment of the \<script\> node
 *
 * @param script \<script\> node instance
 *
 * @returns \<script\> node origin src
 */
function getScriptOriginalSrc(script: HTMLScriptElement): string {
    const scriptUrl = script.src;
    const scriptUrlPolyfill = script.getAttribute("ruffle-src-polyfill");
    if (!scriptUrlPolyfill) {
        return scriptUrl;
    }
    // Reset webkit mask url should be safe
    if ("webkit-masked-url://hidden/" === scriptUrl) {
        try {
            const currentPolyfillURL = new URL(".", scriptUrlPolyfill);
            const isExtensionUrl =
                currentPolyfillURL.protocol.includes("extension");
            // Only apply to extension
            if (isExtensionUrl) {
                return scriptUrlPolyfill;
            }
        } catch (_) {
            // Fallback to itself src
        }
    }
    return scriptUrl;
}
