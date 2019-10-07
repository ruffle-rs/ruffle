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
 * 
 * Defaults mentioned above are not applied by this function.
 */
export function get_config_options(elem) {
    let values = JSON.parse(JSON.stringify(DEFAULT_CONFIG));

    values.optout = elem.dataset.ruffleOptout !== undefined;
    if (elem.dataset.ruffleVersion !== undefined) {
        values.version = elem.dataset.ruffleVersion;
    }

    return values;
}

export const DEFAULT_CONFIG = {
    "optout": false
};