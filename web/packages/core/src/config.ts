/**
 * The configuration object to control Ruffle's behaviour on the website
 * that it is included on.
 */
export interface Config {
    /**
     * A map of public paths from source name to URL.
     */
    public_paths?: Record<string, string>;

    /**
     * The URL at which Ruffle can load its extra files (ie `.wasm`).
     *
     * [public_paths] is consulted first for a source-specific URL,
     * with this field being a fallback.
     */
    public_path?: string;

    /**
     * Whether or not to enable polyfills on the page.
     *
     * Polyfills will look for "legacy" flash content like `<object>`
     * and `<embed>` elements, and replace them with compatible
     * Ruffle elements.
     *
     * @default true
     */
    polyfills?: boolean;
}
