import {
    FLASH_MIMETYPE,
    FUTURESPLASH_MIMETYPE,
    FLASH7_AND_8_MIMETYPE,
    FLASH_MOVIE_MIMETYPE,
} from "./flash-identifiers";

/**
 * Returns whether the given filename ends in a known Flash extension.
 *
 * @param filename The filename to test.
 * @returns True if the filename is a Flash movie (swf or spl).
 */
function isSwfFilename(filename: string): boolean {
    let pathname = "";
    try {
        // A base URL is required if `filename` is a relative URL, but we don't need to detect the real URL origin.
        pathname = new URL(filename, "https://example.com").pathname;
    } catch (_err) {
        // Some invalid filenames, like `///`, could raise a TypeError. Let's fail silently in this situation.
    }
    if (pathname && pathname.length >= 4) {
        const extension = pathname.slice(-4).toLowerCase();
        if (extension === ".swf" || extension === ".spl") {
            return true;
        }
    }
    return false;
}

/**
 * Returns whether the given MIME type is a known Flash type.
 *
 * @param mimeType The MIME type to test.
 * @param allowExtraMimes Whether extra MIME types, non-Flash related, are allowed.
 * @returns True if the MIME type is a Flash MIME type.
 */
function isSwfMimeType(mimeType: string, allowExtraMimes: boolean): boolean {
    mimeType = mimeType.toLowerCase();
    switch (mimeType) {
        case FLASH_MIMETYPE.toLowerCase():
        case FUTURESPLASH_MIMETYPE.toLowerCase():
        case FLASH7_AND_8_MIMETYPE.toLowerCase():
        case FLASH_MOVIE_MIMETYPE.toLowerCase():
            return true;
        default:
            if (allowExtraMimes) {
                // Allow extra MIME types to improve detection of Flash content.
                // Extension: Some sites (e.g. swfchan.net) might (wrongly?) serve files with octet-stream.
                // Polyfill: Other sites (e.g. #11050) might use octet-stream when defining an <embed> tag.
                switch (mimeType) {
                    case "application/octet-stream":
                    case "binary/octet-stream":
                        return true;
                }
            }
    }
    return false;
}

/**
 * Returns whether the given filename and MIME type resolve as a Flash content.
 *
 * @param filename The filename to test.
 * @param mimeType The MIME type to test.
 * @returns True if the given arguments resolve as a Flash content.
 */
export function isSwf(filename: string, mimeType: string | null): boolean {
    const isSwfExtension = isSwfFilename(filename);
    if (!mimeType) {
        // If no MIME type is specified (null or empty string), returns whether the movie ends in a known Flash extension.
        return isSwfExtension;
    } else {
        return isSwfMimeType(mimeType, isSwfExtension);
    }
}

/**
 * Create a filename to save a downloaded SWF into.
 *
 * @param swfUrl The URL of the SWF file.
 * @returns The filename the SWF file can be saved at.
 */
export function swfFileName(swfUrl: URL): string {
    const pathName = swfUrl.pathname;
    const name = pathName.substring(pathName.lastIndexOf("/") + 1);
    return name;
}
