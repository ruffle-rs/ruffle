/**
 * The configuration object to control Ruffle's behaviour on the website
 * that it is included on.
 */
export interface ApiConfig {
    /**
     * The URL at which Ruffle can load its extra files (i.e. `.wasm`).
     *
     * @default null
     */
    publicPath?: string | null;

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

export const DEFAULT_API_CONFIG: Required<ApiConfig> = {
    publicPath: null,
    polyfills: true,
};
