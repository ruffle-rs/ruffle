/**
 * Retrieve the declarative configuration options of the given element.
 * 
 * This is intended to be used with the HTML element of a page. It allows web
 * pages to signal to both self-hosted and WebExtension versions of Ruffle what
 * their authorial intent is.
 * 
 * The following data attributes are recognized:
 * 
 *  * data-ruffle-optout - Signals to the Ruffle Extension that the page would
 *    like to opt out of having Ruffle loaded onto it. Has no effect on the
 *    self-hosted version of Ruffle.
 *  * data-ruffle-version - Indicates that this page self-hosts Ruffle, and
 *    optionally indicates the version of Ruffle present on the page.
 *  * data-ruffle-interdict - Indicates what legacy content interdictions are
 *    allowed. This setting is respected by both WebExtension and self-hosted
 *    Ruffle identically. The default of `static-content,plugin-detect` will be
 *    provided if not specified. The following are valid interdictions:
 *     * static-content - Replace static `<object>` and `<embed>` elements.
 *       Enabled by default.
 *     * dynamic-content - Replace dynamically-added `<object>` and `<embed>`
 *       elements using a `MutationObserver`. Not enabled by default, as this
 *       interdiction is expensive.
 *     * plugin-detect - Alter the `window` in order to fool plugin detects. You
 *       will not be able to detect the native version of Flash dynamically if
 *       enabled.
 * 
 * Defaults mentioned above are not applied by this function.
 */
export function get_config_options(elem) {
    let values = {};

    values.optout = elem.dataset.ruffle-optout !== undefined;
    if (elem.dataset.ruffle-version !== undefined) {
        values.version = elem.dataset.ruffle-version;
    }

    if (elem.dataset.ruffle-interdict !== undefined) {
        values.interdict = elem.dataset.ruffle-interdict.split(",").map((v) => v.trim());
    }
}

export const DEFAULT_CONFIG = {
    "optout": false,
    "interdict": ["static-content", "plugin-detect"]
};