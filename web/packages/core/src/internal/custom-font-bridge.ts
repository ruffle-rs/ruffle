/**
 * Type contract for pluggable font-rasterization backends.
 *
 * A "custom font renderer" is any JS-visible object that satisfies the
 * {@link FontBridge} interface. It plugs in via the global variable
 * `globalThis.__ruffleCustomFontRenderer` when the embedding host
 * selects `deviceFontRenderer: "custom"`; Ruffle then asks the bridge
 * for a {@link CustomFontRenderer} per font family/style and calls
 * methods on that via reflection from the WASM core.
 *
 * Ruffle is agnostic about the technology behind the bridge — a
 * napi-rs native addon, a separate WebAssembly module, a pure
 * JavaScript implementation, anything that satisfies the contract
 * methods will do. The only thing Ruffle sees is a plain object
 * assigned to the global slot; producing that object is entirely the
 * embedding host's responsibility.
 *
 * Because the lookup happens lazily (each time Ruffle needs to
 * materialize a device-font renderer), the bridge object can be
 * assigned after the Ruffle player has already been instantiated — as
 * long as it is in place before the first device-font request. This
 * is useful for bridges shipped as ES modules with top-level await
 * (e.g. wasm-backed renderers), which inevitably load asynchronously.
 *
 * # Two-layer contract
 *
 * The bridge is deliberately narrow: its only job is to manufacture
 * renderer instances for a given font family/style. Everything else —
 * caching native resources across sizes, picking the canonical
 * rasterization size, etc. — lives *inside* the renderer returned by
 * {@link FontBridge.createRenderer}. That means adding a new backend
 * only requires implementing these two interfaces, with no contact
 * with Ruffle's internals.
 *
 * All metrics and coordinates exchanged with the bridge are in **pixel
 * units**. Ruffle converts to its native twips coordinate system on
 * the Rust side; backends never need to know about twips.
 */

/**
 * A rasterized glyph returned by {@link CustomFontRenderer.renderGlyph}.
 *
 * All numeric fields are in pixels. The bitmap is always `width * height`
 * RGBA bytes, row-major, top-down, with premultiplied alpha; non-ink
 * pixels are `(0, 0, 0, 0)`.
 */
export interface GlyphRaster {
    /** Bitmap width in pixels; always `>= 1`. */
    width: number;
    /** Bitmap height in pixels; always `>= 1`. */
    height: number;
    /**
     * Horizontal offset, in pixels, from the current pen position to the
     * bitmap's left edge. Negative when the glyph overhangs to the left
     * of the origin.
     */
    bitmapTx: number;
    /** Advance of the glyph in pixels. */
    advance: number;
    /**
     * RGBA bytes of the bitmap. The length must equal
     * `width * height * 4`.
     */
    pixels: Uint8Array;
}

/**
 * Font-wide metrics returned by
 * {@link CustomFontRenderer.getFontMetrics}.
 *
 * All values are in pixels at the requested rasterization size.
 */
export interface FontMetrics {
    /** Distance from the baseline up to the highest point, in pixels. */
    ascent: number;
    /** Distance from the baseline down to the lowest point, in pixels. */
    descent: number;
    /** External leading, in pixels. Use `0` when unavailable. */
    leading: number;
}

/**
 * A single native font instance, produced by
 * {@link FontBridge.createRenderer}. The backend is free to implement
 * this as a class, a plain object, or anything else that satisfies the
 * method signatures below — Ruffle only invokes its methods through
 * reflection.
 *
 * Lifetime is managed by Ruffle: a renderer is created the first time a
 * given font family/style is needed and is released via
 * {@link CustomFontRenderer.destroy} when Ruffle drops it.
 *
 * Every method is called with a `sizePx` argument where applicable. The
 * backend is expected to manage any internal caching (e.g. one native
 * font handle per requested size) transparently — Ruffle does not cache
 * on its behalf. If a backend cannot honour a given size (e.g. bitmap
 * fonts) it is free to ignore the argument, but the glyphs it returns
 * will then be scaled by the layout pipeline.
 */
export interface CustomFontRenderer {
    /**
     * Whether this renderer can report kerning pairs. If `false`, Ruffle
     * will skip {@link calculateKerning} calls entirely.
     */
    hasKerningInfo(): boolean;
    /**
     * Font-wide metrics when rasterized at `sizePx` pixels.
     */
    getFontMetrics(sizePx: number): FontMetrics;
    /**
     * Rasterize `codePoint` at `sizePx` pixels. Return `null` if the glyph
     * does not exist in the font and the backend cannot even produce an
     * advance for it.
     */
    renderGlyph(codePoint: number, sizePx: number): GlyphRaster | null;
    /**
     * Kerning between `left` and `right` when drawn at `sizePx` pixels,
     * in pixels. Return `0` when the pair has no dedicated kerning.
     */
    calculateKerning(left: number, right: number, sizePx: number): number;
    /**
     * Release every native resource owned by this renderer. Called once,
     * when Ruffle drops the renderer.
     */
    destroy(): void;
}

/**
 * Entry point implemented by a custom font addon. Typically, a napi-rs
 * module simply exports a top-level `createRenderer` function and the
 * module object itself becomes the bridge.
 */
export interface FontBridge {
    /**
     * Spawn a new {@link CustomFontRenderer} for the given family/style,
     * or return `null`/`undefined` if the backend cannot handle it.
     */
    createRenderer(
        family: string,
        bold: boolean,
        italic: boolean,
    ): CustomFontRenderer | null | undefined;

    /**
     * Optional hook invoked once per entry in the Ruffle
     * `fontSources` config after the bytes have been fetched,
     * and before any call to {@link createRenderer}.
     *
     * Bridges that need the raw font bytes to rasterize (typically
     * backends without access to an OS font database, like the swash
     * renderer) should implement this method and use it to parse the
     * data and index the resulting faces by family and style for
     * later lookup in {@link createRenderer}.
     *
     * TTC/OTC collections are delivered as a single call: the bridge
     * is responsible for iterating every face inside the collection.
     *
     * Bridges that resolve fonts through the host operating system
     * (e.g. the GDI addon) can simply omit this method — Ruffle then
     * skips the forwarding step entirely.
     *
     * Errors thrown from this method are logged as warnings and do
     * not abort Ruffle's own font-source processing.
     *
     * @param source The original URL or identifier of the font
     *   source, as configured in `fontSources`. Useful for logging.
     * @param bytes The raw TTF/OTF/TTC/OTC bytes. The bridge must
     *   not mutate this buffer; if it needs to retain it across
     *   calls it should make its own copy.
     */
    registerFontData?(source: string, bytes: Uint8Array): void;
}

/**
 * Well-known name of the global slot where the embedding host must
 * place the bridge object. Kept as a constant so the TS side and any
 * tooling stay in sync with the string the Rust side looks up via
 * reflection (`web/src/ui.rs`).
 */
export const CUSTOM_FONT_RENDERER_GLOBAL = "__ruffleCustomFontRenderer";

/**
 * Structural check: the value must expose `createRenderer` as a
 * function. Everything else is optional and will be probed lazily by
 * the WASM core through reflection.
 *
 * @param value The candidate bridge object to inspect.
 * @returns `true` if `value` is an object exposing a `createRenderer`
 *   function; `false` otherwise.
 */
export function isFontBridge(value: unknown): value is FontBridge {
    if (value === null || typeof value !== "object") {
        return false;
    }
    const obj = value as Record<string, unknown>;
    return typeof obj["createRenderer"] === "function";
}

/**
 * Read `globalThis.__ruffleCustomFontRenderer` and validate it.
 * Returns the bridge if present and structurally valid, otherwise
 * `undefined`. Each caller re-reads the global, so bridges assigned
 * after player construction (e.g. from a deferred `<script type="module">`)
 * are picked up as soon as they become available.
 *
 * @returns The bridge from the global slot, or `undefined` if it is
 *   missing or not structurally a `FontBridge`.
 */
export function getCustomFontRenderer(): FontBridge | undefined {
    const bridge = (globalThis as Record<string, unknown>)[
        CUSTOM_FONT_RENDERER_GLOBAL
    ];
    return isFontBridge(bridge) ? bridge : undefined;
}
