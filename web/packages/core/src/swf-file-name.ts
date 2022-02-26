/**
 * Create a filename to save a downloaded SWF into.
 *
 * @param swfUrl The URL of the SWF file.
 * @returns The filename the SWF file can be saved at.
 */
export function swfFileName(swfUrl: string): string {
    let name = swfUrl.substring(swfUrl.lastIndexOf("/") + 1);
    const qMark = name.indexOf("?");
    if (qMark > 0) {
        name = name.substring(0, qMark);
    }
    return name;
}
