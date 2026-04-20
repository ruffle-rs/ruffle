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

    fn ensure_canvas_large_enough(&self, width: f64, height: f64) {
        let width = width.ceil() as u32;
        let height = height.ceil() as u32;
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
        let bitmap_width = bitmap_width.max(1.0); // TODO Support empty bitmaps.
        let advance = Twips::from_pixels(metrics.width());
        let bitmap_tx = -metrics.actual_bounding_box_left();

        self.ensure_canvas_large_enough(bitmap_width, height);

        self.ctx.clear_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
        self.ctx.fill_text(text, -bitmap_tx, self.ascent)?;

        let image_data = self.ctx.get_image_data(0.0, 0.0, bitmap_width, height)?;
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
