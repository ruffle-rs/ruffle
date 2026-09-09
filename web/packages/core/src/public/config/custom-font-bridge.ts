/**
 * Public type surface for pluggable font-rasterization backends used
 * by the `deviceFontRenderer: "custom"` option together with the
 * `globalThis.__ruffleCustomFontRenderer` global slot.
 *
 * Ruffle never loads bridges on its own: the embedding host installs
 * the bridge object on `globalThis.__ruffleCustomFontRenderer` and
 * Ruffle looks it up lazily whenever a device-font request arrives.
 * These type re-exports exist only to let consumers author such
 * bridges against the expected contract.
 */

export {
    type FontBridge,
    type CustomFontRenderer,
    type FontMetrics,
    type GlyphRaster,
} from "../../internal/custom-font-bridge";
