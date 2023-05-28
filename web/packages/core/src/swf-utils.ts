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
export function isSwfFilename(filename: string | null): boolean {
    if (filename) {
        let pathname = "";
        try {
            // A base URL is required if `filename` is a relative URL, but we don't need to detect the real URL origin.
            pathname = new URL(filename, "https://example.com").pathname;
        } catch (err) {
            // Some invalid filenames, like `///`, could raise a TypeError. Let's fail silently in this situation.
        }
        if (pathname && pathname.length >= 4) {
            const extension = pathname.slice(-4).toLowerCase();
            if (extension === ".swf" || extension === ".spl") {
                return true;
            }
        }
    }
    return false;
}

/**
 * Returns whether the given MIME type is a known Flash type.
 *
 * @param mimeType The MIME type to test.
 * @param allowExtraMimes Whether the polyfill should allow extra MIME types.
 * @returns True if the MIME type is a Flash MIME type.
 */
export function isSwfMimeType(
    mimeType: string,
    allowExtraMimes: boolean
): boolean {
    mimeType = mimeType.toLowerCase();
    switch (mimeType) {
        case FLASH_MIMETYPE.toLowerCase():
        case FUTURESPLASH_MIMETYPE.toLowerCase():
        case FLASH7_AND_8_MIMETYPE.toLowerCase():
        case FLASH_MOVIE_MIMETYPE.toLowerCase():
            return true;
        default:
            if (allowExtraMimes) {
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
