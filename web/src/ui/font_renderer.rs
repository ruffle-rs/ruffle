use js_sys::JSON;
use ruffle_core::font::FontMetrics;
use ruffle_core::font::FontRenderer;
use ruffle_core::font::Glyph;
use ruffle_core::swf::Twips;
use ruffle_render::bitmap::Bitmap;
use ruffle_render::bitmap::BitmapFormat;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::OffscreenCanvas;
use web_sys::OffscreenCanvasRenderingContext2d;

/// Canonical rasterization size, in pixels, used whenever the caller
/// doesn't specify a display size (legacy `render_glyph` path, font
/// metrics reported to the core pipeline). Matches `CanvasFontRenderer`
/// so metrics line up between the two renderers.
const CANONICAL_SIZE_PX: u32 = 64;

/// Twips per pixel (1 px = 20 twips). Shared with `CanvasFontRenderer`.
const TWIPS_PER_PX: f64 = 20.0;

/// Canonical em-scale in twips (`CANONICAL_SIZE_PX * TWIPS_PER_PX`).
const CANONICAL_EM_SCALE: f32 = CANONICAL_SIZE_PX as f32 * TWIPS_PER_PX as f32;

#[derive(Debug)]
pub struct CanvasFontRenderer {
    canvas: OffscreenCanvas,
    ctx: OffscreenCanvasRenderingContext2d,
    font_str: String,
    ascent: f64,
    descent: f64,
}

impl CanvasFontRenderer {
    /// Render fonts with size 64px. It affects the bitmap size.
    const SIZE_PX: f64 = 64.0;

    /// Divide each pixel into 20 (use twips precision). It affects metrics.
    const SCALE: f64 = 20.0;

    pub fn new(italic: bool, bold: bool, font_family: &str) -> Result<Self, JsValue> {
        if !Self::is_offscreen_canvas_supported() {
            return Err(JsValue::from_str("OffscreenCanvas unsupported"));
        }

        let canvas = OffscreenCanvas::new(1, 1)?;

        let ctx = canvas.get_context("2d")?.expect("2d context");
        let ctx = ctx
            .dyn_into::<OffscreenCanvasRenderingContext2d>()
            .map_err(|err| JsValue::from_str(&format!("Not a 2d context: {err:?}")))?;

        let font_str = Self::to_font_str(italic, bold, Self::SIZE_PX, font_family);
        tracing::debug!("Using the following font string: {font_str}");
        Self::apply_style(&ctx, &font_str);

        let measurement = ctx.measure_text("Myjg")?;
        let ascent = measurement.font_bounding_box_ascent();
        let descent = measurement.font_bounding_box_descent();

        Ok(Self {
            canvas,
            ctx,
            font_str,
            ascent,
            descent,
        })
    }

    // TODO Remove it when we stop supporting Firefox <105, Safari <16.4
    fn is_offscreen_canvas_supported() -> bool {
        let global = js_sys::global();
        match js_sys::Reflect::get(&global, &JsValue::from_str("OffscreenCanvas")) {
            Ok(value) => !value.is_undefined(),
            Err(_) => false,
        }
    }

    fn to_font_str(italic: bool, bold: bool, size: f64, font_family: &str) -> String {
        let italic = if italic { "italic " } else { "" };
        let bold = if bold { "bold " } else { "" };

        // Escape font family properly
        let font_family = JSON::stringify(&JsValue::from_str(font_family))
            .ok()
            .and_then(|js_str| js_str.as_string())
            .unwrap_or_else(|| format!("\"{font_family}\""));
        format!("{italic}{bold}{size}px {font_family}")
    }

    fn apply_style(ctx: &OffscreenCanvasRenderingContext2d, font_str: &str) {
        ctx.set_fill_style_str("white");
        ctx.set_font(font_str);
    }

    fn calculate_width(&self, text: &str) -> Result<f64, JsValue> {
        Ok(self.ctx.measure_text(text)?.width())
    }

    fn ensure_canvas_large_enough(&self, width: u32, height: u32) {
        if self.canvas.width() < width || self.canvas.height() < height {
            self.canvas.set_width(width);
            self.canvas.set_height(height);

            // After changing canvas size, we need to reapply the style.
            // Somehow, when canvas size is too small for the text, the
            // text is rendered smaller, but its reported metrics are correct.
            Self::apply_style(&self.ctx, &self.font_str);
        }
    }

    fn render_glyph_internal(&self, character: char) -> Result<Glyph, JsValue> {
        let text = &character.to_string();
        let metrics = self.ctx.measure_text(text)?;
        let height = self.ascent + self.descent;

        let bitmap_width = metrics.actual_bounding_box_left() + metrics.actual_bounding_box_right();
        let bitmap_width = bitmap_width.max(1.0).ceil() as i32; // TODO Support empty bitmaps.
        let bitmap_height = height.max(1.0).ceil() as i32; // TODO Support empty bitmaps.
        let advance = Twips::from_pixels(metrics.width());
        let bitmap_tx = -metrics.actual_bounding_box_left();

        self.ensure_canvas_large_enough(bitmap_width as u32, bitmap_height as u32);

        self.ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        self.ctx.fill_text(text, -bitmap_tx, self.ascent)?;

        let image_data = self.ctx.get_image_data(0, 0, bitmap_width, bitmap_height)?;
        let width = image_data.width();
        let height = image_data.height();
        let pixels = image_data.data().0;

        let bitmap = Bitmap::new(width, height, BitmapFormat::Rgba, pixels);
        let bitmap_tx = Twips::from_pixels(-metrics.actual_bounding_box_left());
        Ok(Glyph::from_bitmap(character, bitmap, advance, bitmap_tx))
    }

    fn calculate_kerning_internal(&self, left: char, right: char) -> Result<Twips, JsValue> {
        let left_width = self.calculate_width(&left.to_string())?;
        let right_width = self.calculate_width(&right.to_string())?;
        let both_width = self.calculate_width(&format!("{left}{right}"))?;

        let kern = both_width - left_width - right_width;
        Ok(Twips::from_pixels(kern))
    }
}

impl FontRenderer for CanvasFontRenderer {
    fn scale(&self) -> f32 {
        (Self::SIZE_PX * Self::SCALE) as f32
    }

    fn get_font_metrics(&self) -> FontMetrics {
        FontMetrics {
            scale: self.scale(),
            ascent: (self.ascent * Self::SCALE) as i32,
            descent: (self.descent * Self::SCALE) as i32,
            leading: 0,
        }
    }

    fn has_kerning_info(&self) -> bool {
        true
    }

    fn render_glyph(&self, character: char) -> Option<Glyph> {
        self.render_glyph_internal(character)
            .map_err(|err| tracing::error!("Failed to render a glyph: {err:?}"))
            .ok()
    }

    fn calculate_kerning(&self, left: char, right: char) -> Twips {
        self.calculate_kerning_internal(left, right)
            .map_err(|err| tracing::error!("Failed to calculate kerning: {err:?}"))
            .unwrap_or(Twips::ZERO)
    }
}

/// Device-font renderer that delegates glyph rasterization to a
/// pluggable `FontBridge` object supplied by the embedding host.
///
/// The WASM side is agnostic about how the bridge is produced — a
/// napi-rs native addon, a separate WebAssembly module, a pure
/// JavaScript polyfill, anything exposing the contract will do. The
/// bridge `JsValue` is fetched from
/// `globalThis.__ruffleCustomFontRenderer` on every font request by
/// `WebUiBackend::load_device_font` and passed to this struct's
/// constructor; all subsequent calls are made via reflection. See
/// `web/packages/core/src/internal/custom-font-bridge.ts` for the TS
/// types and `web/src/ui.rs` for the plumbing.
///
/// This struct is intentionally thin: every expensive concern — native
/// font handle pooling, size-aware caching, font-specific fallbacks —
/// is owned by the bridge. Ruffle only:
///   1. asks the bridge for a per-font-style renderer at construction,
///   2. caches the canonical-size metrics it needs for `FontMetrics`,
///   3. forwards each glyph request to JS and translates pixel units
///      into Ruffle's twips coordinate system.
///
/// Rasterization size follows the caller: the size-aware trait methods
/// pass through the requested pixel height, and the resulting `Glyph`
/// carries an `intrinsic_scale` that collapses the layout-time scale to
/// 1.0 so the native pixels hit the screen 1:1.
pub struct CustomFontRenderer {
    /// Renderer object produced by `bridge.createRenderer(...)`. All
    /// other methods are invoked on this via reflection.
    js_renderer: JsValue,
    /// Cached font metrics (at `CANONICAL_SIZE_PX`) in twips.
    ascent_twips: i32,
    descent_twips: i32,
    leading_twips: i16,
    /// Cached `hasKerningInfo()` result; avoids a JS call per kerning pair.
    has_kerning: bool,
}

impl CustomFontRenderer {
    pub fn new(
        italic: bool,
        bold: bool,
        font_family: &str,
        bridge: &JsValue,
    ) -> Result<Self, JsValue> {
        let js_renderer = Self::call_fn(
            bridge,
            "createRenderer",
            &[
                JsValue::from_str(font_family),
                JsValue::from_bool(bold),
                JsValue::from_bool(italic),
            ],
        )?;
        if js_renderer.is_null() || js_renderer.is_undefined() {
            return Err(JsValue::from_str(
                "Custom font bridge createRenderer() returned null",
            ));
        }

        // Cache the canonical-size metrics and the kerning-support flag so
        // we don't have to cross the JS boundary every time the core asks.
        let metrics = Self::call_fn(
            &js_renderer,
            "getFontMetrics",
            &[JsValue::from_f64(CANONICAL_SIZE_PX as f64)],
        )?;
        let ascent_px = Self::get_number(&metrics, "ascent")?;
        let descent_px = Self::get_number(&metrics, "descent")?;
        let leading_px = Self::get_number(&metrics, "leading").unwrap_or(0.0);

        let has_kerning = Self::call_fn(&js_renderer, "hasKerningInfo", &[])?
            .as_bool()
            .unwrap_or(false);

        Ok(Self {
            js_renderer,
            ascent_twips: (ascent_px * TWIPS_PER_PX) as i32,
            descent_twips: (descent_px * TWIPS_PER_PX) as i32,
            leading_twips: (leading_px * TWIPS_PER_PX) as i16,
            has_kerning,
        })
    }

    fn call_fn(receiver: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
        let func_val = js_sys::Reflect::get(receiver, &JsValue::from_str(name))?;
        let func: js_sys::Function = func_val
            .dyn_into()
            .map_err(|_| JsValue::from_str(&format!("{name} is not a function")))?;
        let args_array = js_sys::Array::new();
        for a in args {
            args_array.push(a);
        }
        func.apply(receiver, &args_array)
    }

    fn get_number(obj: &JsValue, key: &str) -> Result<f64, JsValue> {
        js_sys::Reflect::get(obj, &JsValue::from_str(key))?
            .as_f64()
            .ok_or_else(|| JsValue::from_str(&format!("expected number for \"{key}\"")))
    }

    /// Effective rasterization size: caller's `height_px`, falling back
    /// to the canonical size when the caller did not specify one (`0`).
    fn effective_size(height_px: u32) -> u32 {
        if height_px == 0 {
            CANONICAL_SIZE_PX
        } else {
            height_px
        }
    }

    fn render_glyph_internal(
        &self,
        character: char,
        size_px: u32,
    ) -> Result<Option<Glyph>, JsValue> {
        let result = Self::call_fn(
            &self.js_renderer,
            "renderGlyph",
            &[
                JsValue::from_f64(character as u32 as f64),
                JsValue::from_f64(size_px as f64),
            ],
        )?;
        if result.is_null() || result.is_undefined() {
            return Ok(None);
        }

        let width = Self::get_number(&result, "width")? as u32;
        let height = Self::get_number(&result, "height")? as u32;
        let bitmap_tx_px = Self::get_number(&result, "bitmapTx")?;
        let advance_px = Self::get_number(&result, "advance")?;
        let pixels_val = js_sys::Reflect::get(&result, &JsValue::from_str("pixels"))?;
        let pixels_u8 = js_sys::Uint8Array::new(&pixels_val);
        let pixels = pixels_u8.to_vec();

        if (pixels.len() as u32) != width * height * 4 {
            return Err(JsValue::from_str(
                "Glyph pixels buffer size does not match width*height*4",
            ));
        }

        let bitmap = Bitmap::new(width, height, BitmapFormat::Rgba, pixels);
        let advance = Twips::from_pixels(advance_px);
        let bitmap_tx = Twips::from_pixels(bitmap_tx_px);

        // `intrinsic_scale` is the em-scale at which this glyph was
        // rasterized, in twips. When the layout asks for
        // `height_twips == size_px * TWIPS_PER_PX`, `evaluate()` sees
        // `scale = height / intrinsic_scale = 1.0`, so the native pixels
        // produced by the pluggable renderer land 1:1 on the screen.
        let intrinsic_scale = size_px as f32 * TWIPS_PER_PX as f32;
        Ok(Some(Glyph::from_bitmap_at_scale(
            character,
            bitmap,
            advance,
            bitmap_tx,
            intrinsic_scale,
        )))
    }

    fn font_metrics_internal(&self, size_px: u32) -> Result<FontMetrics, JsValue> {
        let metrics = Self::call_fn(
            &self.js_renderer,
            "getFontMetrics",
            &[JsValue::from_f64(size_px as f64)],
        )?;
        let ascent_px = Self::get_number(&metrics, "ascent")?;
        let descent_px = Self::get_number(&metrics, "descent")?;
        let leading_px = Self::get_number(&metrics, "leading").unwrap_or(0.0);
        // `scale` equals the raster size in twips: when the layout asks for
        // this same size, `FontMetrics::ascent/descent` come out exactly as
        // the bridge reported them — whole device pixels for pixel-locked
        // backends like GDI — instead of a canonical-size measurement
        // scaled into a fractional value.
        Ok(FontMetrics {
            scale: size_px as f32 * TWIPS_PER_PX as f32,
            ascent: (ascent_px * TWIPS_PER_PX) as i32,
            descent: (descent_px * TWIPS_PER_PX) as i32,
            leading: (leading_px * TWIPS_PER_PX) as i16,
        })
    }

    /// Typographic metrics (`typoAscent`/`typoDescent` from `getFontMetrics`),
    /// if the renderer supplies them. `Ok(None)` when the fields are absent, so
    /// the Flash Text Engine falls back to the cell metrics.
    fn typo_font_metrics_internal(&self, size_px: u32) -> Result<Option<FontMetrics>, JsValue> {
        let metrics = Self::call_fn(
            &self.js_renderer,
            "getFontMetrics",
            &[JsValue::from_f64(size_px as f64)],
        )?;
        let (Ok(ascent_px), Ok(descent_px)) = (
            Self::get_number(&metrics, "typoAscent"),
            Self::get_number(&metrics, "typoDescent"),
        ) else {
            return Ok(None);
        };
        Ok(Some(FontMetrics {
            scale: size_px as f32 * TWIPS_PER_PX as f32,
            ascent: (ascent_px * TWIPS_PER_PX) as i32,
            descent: (descent_px * TWIPS_PER_PX) as i32,
            leading: 0,
        }))
    }

    fn calculate_kerning_internal(
        &self,
        left: char,
        right: char,
        size_px: u32,
    ) -> Result<Twips, JsValue> {
        let kern_px = Self::call_fn(
            &self.js_renderer,
            "calculateKerning",
            &[
                JsValue::from_f64(left as u32 as f64),
                JsValue::from_f64(right as u32 as f64),
                JsValue::from_f64(size_px as f64),
            ],
        )?
        .as_f64()
        .unwrap_or(0.0);
        Ok(Twips::from_pixels(kern_px))
    }
}

impl Drop for CustomFontRenderer {
    fn drop(&mut self) {
        // Best effort: hand control back to the addon so it can release
        // native resources. Any failure is logged but otherwise ignored.
        if let Err(err) = Self::call_fn(&self.js_renderer, "destroy", &[]) {
            tracing::warn!("Custom font renderer destroy() failed: {err:?}");
        }
    }
}

impl std::fmt::Debug for CustomFontRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomFontRenderer")
            .field("ascent_twips", &self.ascent_twips)
            .field("descent_twips", &self.descent_twips)
            .field("leading_twips", &self.leading_twips)
            .field("has_kerning", &self.has_kerning)
            .finish()
    }
}

impl FontRenderer for CustomFontRenderer {
    fn scale(&self) -> f32 {
        CANONICAL_EM_SCALE
    }

    fn get_font_metrics(&self) -> FontMetrics {
        FontMetrics {
            scale: CANONICAL_EM_SCALE,
            ascent: self.ascent_twips,
            descent: self.descent_twips,
            leading: self.leading_twips,
        }
    }

    fn has_kerning_info(&self) -> bool {
        self.has_kerning
    }

    fn render_glyph(&self, character: char) -> Option<Glyph> {
        // Unsized path: render at the canonical size.
        match self.render_glyph_internal(character, CANONICAL_SIZE_PX) {
            Ok(glyph) => glyph,
            Err(err) => {
                tracing::error!(
                    "Custom font renderer renderGlyph failed for {character:?}: {err:?}",
                );
                None
            }
        }
    }

    fn calculate_kerning(&self, left: char, right: char) -> Twips {
        self.calculate_kerning_internal(left, right, CANONICAL_SIZE_PX)
            .map_err(|err| tracing::error!("Custom font renderer calculateKerning failed: {err:?}"))
            .unwrap_or(Twips::ZERO)
    }

    fn render_glyph_at_size(&self, character: char, height_px: u32) -> Option<Glyph> {
        let size = Self::effective_size(height_px);
        match self.render_glyph_internal(character, size) {
            Ok(glyph) => glyph,
            Err(err) => {
                tracing::error!(
                    "Custom font renderer renderGlyph failed for {character:?} @ {height_px}px: {err:?}",
                );
                None
            }
        }
    }

    fn calculate_kerning_at_size(&self, left: char, right: char, height_px: u32) -> Twips {
        let size = Self::effective_size(height_px);
        self.calculate_kerning_internal(left, right, size)
            .map_err(|err| {
                tracing::error!(
                    "Custom font renderer calculateKerning failed @ {height_px}px: {err:?}",
                )
            })
            .unwrap_or(Twips::ZERO)
    }

    fn get_font_metrics_at_size(&self, height_px: u32) -> Option<FontMetrics> {
        let size = Self::effective_size(height_px);
        match self.font_metrics_internal(size) {
            Ok(metrics) => Some(metrics),
            Err(err) => {
                tracing::error!("Custom font renderer getFontMetrics failed @ {size}px: {err:?}");
                None
            }
        }
    }

    fn get_typo_font_metrics(&self, height_px: u32) -> Option<FontMetrics> {
        let size = Self::effective_size(height_px);
        match self.typo_font_metrics_internal(size) {
            Ok(metrics) => metrics,
            Err(err) => {
                tracing::error!(
                    "Custom font renderer getFontMetrics (typo) failed @ {size}px: {err:?}"
                );
                None
            }
        }
    }

    fn is_size_aware(&self) -> bool {
        true
    }
}
