use crate::utils::JsResult;
use ruffle_core::backend::render::{
    swf, swf::CharacterId, BitmapHandle, BitmapInfo, Color, Letterbox, RenderBackend, ShapeHandle,
    Transform,
};
use ruffle_core::color_transform::ColorTransform;
use ruffle_core::shape_utils::DrawCommand;
use std::collections::HashMap;
use std::convert::TryInto;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    CanvasGradient, CanvasPattern, CanvasRenderingContext2d, Element, HtmlCanvasElement,
    HtmlImageElement, Path2d, SvgsvgElement,
};

pub struct WebCanvasRenderBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    root_canvas: HtmlCanvasElement,
    render_targets: Vec<(HtmlCanvasElement, CanvasRenderingContext2d)>,
    cur_render_target: usize,
    color_matrix: Element,
    shapes: Vec<ShapeData>,
    bitmaps: Vec<BitmapData>,
    id_to_bitmap: HashMap<CharacterId, BitmapHandle>,
    viewport_width: u32,
    viewport_height: u32,
    use_color_transform_hack: bool,
    pixelated_property_value: &'static str,
}

/// Canvas-drawable shape data extracted from an SWF file.
struct ShapeData(Vec<CanvasDrawCommand>);

struct CanvasColor(String, u8, u8, u8, u8);

/// Convert an f32 to a u8, clamping all out-of-range values to the `u8` range.
fn clamped_u8_color(v: f32) -> u8 {
    if v < 0.0 {
        0
    } else if v > 255.0 {
        255
    } else {
        v as u8
    }
}

impl CanvasColor {
    /// Apply a color transformation to this color.
    fn color_transform(&self, cxform: &ColorTransform) -> CanvasColor {
        let CanvasColor(_, r, g, b, a) = self;
        let r = clamped_u8_color(*r as f32 * cxform.r_mult + (cxform.r_add * 255.0));
        let g = clamped_u8_color(*g as f32 * cxform.g_mult + (cxform.g_add * 255.0));
        let b = clamped_u8_color(*b as f32 * cxform.b_mult + (cxform.b_add * 255.0));
        let a = clamped_u8_color(*a as f32 * cxform.a_mult + (cxform.a_add * 255.0));
        let colstring = format!("rgba({},{},{},{})", r, g, b, f32::from(a) / 255.0);
        CanvasColor(colstring, r, g, b, a)
    }
}

/// An individual command to be drawn to the canvas.
enum CanvasDrawCommand {
    /// A command to draw a path stroke with a given style.
    Stroke {
        path: Path2d,
        line_width: f64,
        stroke_style: CanvasColor,
        line_cap: String,
        line_join: String,
        miter_limit: f64,
    },

    /// A command to fill a path with a given style.
    Fill {
        path: Path2d,
        fill_style: CanvasFillStyle,
    },

    /// A command to draw a particular image (such as an SVG)
    DrawImage {
        image: HtmlImageElement,
        x_min: f64,
        y_min: f64,
    },
}

enum CanvasFillStyle {
    Color(CanvasColor),
    #[allow(dead_code)]
    Gradient(CanvasGradient),
    Pattern(CanvasPattern),
}

impl CanvasFillStyle {
    /// Attempt to apply a color transformation to this fill style.
    fn color_transform(&self, cxform: &ColorTransform) -> Option<CanvasFillStyle> {
        match self {
            Self::Color(cc) => Some(Self::Color(cc.color_transform(cxform))),
            _ => None,
        }
    }
}

#[allow(dead_code)]
struct BitmapData {
    image: HtmlImageElement,
    width: u32,
    height: u32,
    data: String,
}

impl WebCanvasRenderBackend {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, Box<dyn std::error::Error>> {
        // Request the CanvasRenderingContext2d.
        // Disable alpha for possible speedup.
        // TODO: Allow user to enable transparent background (transparent wmode in legacy Flash).
        let context_options = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &context_options,
            &"alpha".into(),
            &wasm_bindgen::JsValue::FALSE,
        );
        let context: CanvasRenderingContext2d = canvas
            .get_context_with_context_options("2d", &context_options)
            .into_js_result()?
            .ok_or("Could not create context")?
            .dyn_into()
            .map_err(|_| "Expected CanvasRenderingContext2d")?;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        // Create a color matrix filter to handle Flash color effects.
        // We may have a previous instance if this canvas was re-used, so remove it.
        if let Ok(Some(element)) = canvas.query_selector("#_svgfilter") {
            element.remove();
        }

        let svg = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
            .map_err(|_| "Couldn't make SVG")?;

        svg.set_id("_svgfilter");

        svg.set_attribute("width", "0")
            .map_err(|_| "Couldn't make SVG")?;

        svg.set_attribute("height", "0")
            .map_err(|_| "Couldn't make SVG")?;

        svg.set_attribute_ns(
            Some("http://www.w3.org/2000/xmlns/"),
            "xmlns:xlink",
            "http://www.w3.org/1999/xlink",
        )
        .map_err(|_| "Couldn't make SVG")?;

        let filter = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "filter")
            .map_err(|_| "Couldn't make SVG filter")?;
        filter
            .set_attribute("id", "_cm")
            .map_err(|_| "Couldn't make SVG filter")?;
        filter
            .set_attribute("color-interpolation-filters", "sRGB")
            .map_err(|_| "Couldn't make SVG filter")?;

        let color_matrix = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "feColorMatrix")
            .map_err(|_| "Couldn't make SVG feColorMatrix element")?;
        color_matrix
            .set_attribute("type", "matrix")
            .map_err(|_| "Couldn't make SVG feColorMatrix element")?;
        color_matrix
            .set_attribute("values", "1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 0")
            .map_err(|_| "Couldn't make SVG feColorMatrix element")?;

        filter
            .append_child(&color_matrix.clone())
            .map_err(|_| "append_child failed")?;

        svg.append_child(&filter)
            .map_err(|_| "append_child failed")?;

        canvas
            .append_child(&svg)
            .map_err(|_| "append_child failed")?;

        // Check if we are on Firefox to use the color transform hack.
        // TODO: We could turn this into a general util function to detect browser
        // type, version, OS, etc.
        let is_firefox = window
            .navigator()
            .user_agent()
            .map(|s| s.contains("Firefox"))
            .unwrap_or(false);

        let render_targets = vec![(canvas.clone(), context.clone())];
        let renderer = Self {
            canvas: canvas.clone(),
            root_canvas: canvas.clone(),
            render_targets,
            cur_render_target: 0,
            color_matrix,
            context,
            shapes: vec![],
            bitmaps: vec![],
            id_to_bitmap: HashMap::new(),
            viewport_width: 0,
            viewport_height: 0,
            use_color_transform_hack: is_firefox,

            // For rendering non-smoothed bitmaps.
            // crisp-edges works in Firefox, pixelated works in Chrome (and others)?
            pixelated_property_value: if is_firefox {
                "crisp-edges"
            } else {
                "pixelated"
            },
        };
        Ok(renderer)
    }

    /// Converts an RGBA image into a PNG encoded as a base64 data URI.
    fn rgba_to_png_data_uri(
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use png::Encoder;
        let mut png_data: Vec<u8> = vec![];
        {
            let mut encoder = Encoder::new(&mut png_data, width, height);
            encoder.set_color(png::ColorType::RGBA);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(&rgba)?;
        }

        Ok(format!(
            "data:image/png;base64,{}",
            &base64::encode(&png_data[..])
        ))
    }

    // Pushes a fresh canvas onto the stack to use as a render target.
    fn push_render_target(&mut self) {
        self.cur_render_target += 1;
        if self.cur_render_target >= self.render_targets.len() {
            // Create offscreen canvas to use as the render target.
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas: HtmlCanvasElement = document
                .create_element("canvas")
                .unwrap()
                .dyn_into()
                .unwrap();
            let context: CanvasRenderingContext2d = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();
            canvas
                .style()
                .set_property("display", "none")
                .warn_on_error();
            self.root_canvas.append_child(&canvas).warn_on_error();
            self.render_targets.push((canvas, context));
        }

        let (canvas, context) = &self.render_targets[self.cur_render_target];
        canvas.set_width(self.viewport_width);
        canvas.set_height(self.viewport_height);
        self.canvas = canvas.clone();
        self.context = context.clone();
        let width = self.canvas.width();
        let height = self.canvas.height();
        self.context
            .clear_rect(0.0, 0.0, width.into(), height.into());
    }

    fn pop_render_target(&mut self) -> (HtmlCanvasElement, CanvasRenderingContext2d) {
        if self.cur_render_target > 0 {
            let out = (self.canvas.clone(), self.context.clone());
            self.cur_render_target -= 1;
            let (canvas, context) = &self.render_targets[self.cur_render_target];
            self.canvas = canvas.clone();
            self.context = context.clone();
            out
        } else {
            log::error!("Render target stack underflow");
            (self.canvas.clone(), self.context.clone())
        }
    }

    #[allow(clippy::float_cmp)]
    #[inline]
    fn set_transform(&mut self, transform: &Transform) {
        let matrix = transform.matrix;

        self.context
            .set_transform(
                matrix.a.into(),
                matrix.b.into(),
                matrix.c.into(),
                matrix.d.into(),
                matrix.tx.to_pixels(),
                matrix.ty.to_pixels(),
            )
            .unwrap();
    }

    #[allow(clippy::float_cmp)]
    #[inline]
    fn set_color_filter(&self, transform: &Transform) {
        let color_transform = &transform.color_transform;
        if color_transform.r_mult == 1.0
            && color_transform.g_mult == 1.0
            && color_transform.b_mult == 1.0
            && color_transform.r_add == 0.0
            && color_transform.g_add == 0.0
            && color_transform.b_add == 0.0
            && color_transform.a_add == 0.0
        {
            self.context.set_global_alpha(color_transform.a_mult.into());
        } else {
            // TODO HACK: Firefox is having issues with additive alpha in color transforms (see #38).
            // Hack this away and just use multiplicative (not accurate in many cases, but won't look awful).
            let (a_mult, a_add) = if self.use_color_transform_hack && color_transform.a_add != 0.0 {
                (color_transform.a_mult + color_transform.a_add, 0.0)
            } else {
                (color_transform.a_mult, color_transform.a_add)
            };

            let matrix_str = format!(
                "{} 0 0 0 {} 0 {} 0 0 {} 0 0 {} 0 {} 0 0 0 {} {}",
                color_transform.r_mult,
                color_transform.r_add,
                color_transform.g_mult,
                color_transform.g_add,
                color_transform.b_mult,
                color_transform.b_add,
                a_mult,
                a_add
            );

            self.color_matrix
                .set_attribute("values", &matrix_str)
                .unwrap();

            self.context.set_filter("url('#_cm')");
        }
    }

    #[inline]
    fn clear_color_filter(&self) {
        self.context.set_filter("none");
        self.context.set_global_alpha(1.0);
    }
}

impl RenderBackend for WebCanvasRenderBackend {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.shapes.len());

        let mut bitmaps = HashMap::new();
        for (id, handle) in &self.id_to_bitmap {
            let bitmap_data = &self.bitmaps[handle.0];
            bitmaps.insert(
                *id,
                (&bitmap_data.data[..], bitmap_data.width, bitmap_data.height),
            );
        }

        let data = swf_shape_to_canvas_commands(
            &shape,
            &bitmaps,
            self.pixelated_property_value,
            &self.context,
        )
        .unwrap_or_else(|| swf_shape_to_svg(&shape, &bitmaps, self.pixelated_property_value));

        self.shapes.push(data);

        handle
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        // Per SWF19 p.164, the FontBoundsTable can contain empty bounds for every glyph (reserved).
        // SWF19 says this is true through SWFv7, but it seems like it might be generally true?
        // In any case, we have to be sure to calculate the shape bounds ourselves to make a proper
        // SVG.
        let bounds = glyph
            .clone()
            .bounds
            .filter(|b| b.x_min != b.x_max || b.y_min != b.y_max)
            .unwrap_or_else(|| {
                ruffle_core::shape_utils::calculate_shape_bounds(&glyph.shape_records[..])
            });
        let shape = swf::Shape {
            version: 2,
            id: 0,
            shape_bounds: bounds.clone(),
            edge_bounds: bounds,
            has_fill_winding_rule: false,
            has_non_scaling_strokes: false,
            has_scaling_strokes: true,
            styles: swf::ShapeStyles {
                fill_styles: vec![swf::FillStyle::Color(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                })],
                line_styles: vec![],
            },
            shape: glyph.shape_records.clone(),
        };
        self.register_shape(&shape)
    }

    fn register_bitmap_jpeg(
        &mut self,
        id: CharacterId,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> BitmapInfo {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_jpeg_2(id, &data[..])
    }

    fn register_bitmap_jpeg_2(&mut self, id: CharacterId, data: &[u8]) -> BitmapInfo {
        let data = ruffle_core::backend::render::remove_invalid_jpeg_data(data);
        let mut decoder = jpeg_decoder::Decoder::new(&data[..]);
        decoder.read_info().unwrap();
        let metadata = decoder.info().unwrap();

        let image = HtmlImageElement::new().unwrap();
        let jpeg_encoded = format!("data:image/jpeg;base64,{}", &base64::encode(&data[..]));
        image.set_src(&jpeg_encoded);

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            image,
            width: metadata.width.into(),
            height: metadata.height.into(),
            data: jpeg_encoded,
        });
        self.id_to_bitmap.insert(id, handle);
        BitmapInfo {
            handle,
            width: metadata.width,
            height: metadata.height,
        }
    }

    fn register_bitmap_jpeg_3(
        &mut self,
        id: swf::CharacterId,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> BitmapInfo {
        let (width, height, mut rgba) =
            ruffle_core::backend::render::define_bits_jpeg_to_rgba(jpeg_data, alpha_data)
                .expect("Error decoding DefineBitsJPEG3");

        ruffle_core::backend::render::unmultiply_alpha_rgba(&mut rgba[..]);
        let png = Self::rgba_to_png_data_uri(&rgba[..], width, height).unwrap();

        let image = HtmlImageElement::new().unwrap();
        image.set_src(&png);

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            image,
            width,
            height,
            data: png,
        });

        self.id_to_bitmap.insert(id, handle);
        BitmapInfo {
            handle,
            width: width.try_into().expect("JPEG dimensions too large"),
            height: height.try_into().expect("JPEG dimensions too large"),
        }
    }

    fn register_bitmap_png(&mut self, swf_tag: &swf::DefineBitsLossless) -> BitmapInfo {
        let mut rgba = ruffle_core::backend::render::define_bits_lossless_to_rgba(swf_tag)
            .expect("Error decoding DefineBitsLossless");

        ruffle_core::backend::render::unmultiply_alpha_rgba(&mut rgba[..]);

        let png =
            Self::rgba_to_png_data_uri(&rgba[..], swf_tag.width.into(), swf_tag.height.into())
                .unwrap();

        let image = HtmlImageElement::new().unwrap();
        image.set_src(&png);

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            image,
            width: swf_tag.width.into(),
            height: swf_tag.height.into(),
            data: png,
        });
        self.id_to_bitmap.insert(swf_tag.id, handle);
        BitmapInfo {
            handle,
            width: swf_tag.width,
            height: swf_tag.height,
        }
    }

    fn begin_frame(&mut self) {
        // Reset canvas transform in case it was left in a dirty state.
        self.context.reset_transform().unwrap();
    }

    fn end_frame(&mut self) {
        // Noop
    }

    fn clear(&mut self, color: Color) {
        let width = self.canvas.width();
        let height = self.canvas.height();

        let color = format!("rgb({}, {}, {})", color.r, color.g, color.b);
        self.context.set_fill_style(&color.into());
        self.context
            .fill_rect(0.0, 0.0, width.into(), height.into());
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform) {
        self.set_transform(transform);
        self.set_color_filter(transform);
        if let Some(bitmap) = self.bitmaps.get(bitmap.0) {
            let _ = self
                .context
                .draw_image_with_html_image_element(&bitmap.image, 0.0, 0.0);
        }
        self.clear_color_filter();
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        self.set_transform(transform);
        if let Some(shape) = self.shapes.get(shape.0) {
            for command in shape.0.iter() {
                match command {
                    CanvasDrawCommand::Fill { path, fill_style } => {
                        let xformed_fill_style =
                            fill_style.color_transform(&transform.color_transform);
                        if xformed_fill_style.is_none() {
                            self.set_color_filter(transform);
                        }

                        match xformed_fill_style.as_ref().unwrap_or(fill_style) {
                            CanvasFillStyle::Color(CanvasColor(color, ..)) => {
                                self.context.set_fill_style(&JsValue::from_str(&color))
                            }
                            CanvasFillStyle::Gradient(grad) => self.context.set_fill_style(grad),
                            CanvasFillStyle::Pattern(patt) => self.context.set_fill_style(patt),
                        };

                        self.context.fill_with_path_2d(&path);

                        if xformed_fill_style.is_none() {
                            self.clear_color_filter();
                        }
                    }
                    CanvasDrawCommand::Stroke {
                        path,
                        line_width,
                        stroke_style,
                        line_cap,
                        line_join,
                        miter_limit,
                    } => {
                        let xformed_stroke_style =
                            stroke_style.color_transform(&transform.color_transform);
                        self.context.set_line_width(*line_width);
                        self.context.set_line_cap(&line_cap);
                        self.context.set_line_join(&line_join);
                        self.context.set_miter_limit(*miter_limit);
                        self.context
                            .set_stroke_style(&JsValue::from_str(&xformed_stroke_style.0));
                        self.context.stroke_with_path(&path);
                    }
                    CanvasDrawCommand::DrawImage {
                        image,
                        x_min,
                        y_min,
                    } => {
                        self.set_color_filter(transform);
                        let _ = self
                            .context
                            .draw_image_with_html_image_element(&image, *x_min, *y_min);
                        self.clear_color_filter();
                    }
                }
            }
        }
    }

    fn draw_pause_overlay(&mut self) {
        let width = f64::from(self.canvas.width());
        let height = f64::from(self.canvas.height());
        self.context.set_fill_style(&"rgba(0, 0, 0, 0.5)".into());
        self.context.fill_rect(0.0, 0.0, width, height);
        self.context.set_text_align("center");
        self.context.set_fill_style(&"white".into());
        self.context
            .set_font(&format!("bold {}px sans-serif", height * 0.1));
        let _ = self
            .context
            .fill_text("Click to Play", width / 2.0, height / 2.0);
    }

    fn draw_letterbox(&mut self, letterbox: Letterbox) {
        self.context.reset_transform().unwrap();
        self.context.set_fill_style(&"black".into());

        match letterbox {
            Letterbox::None => (),
            Letterbox::Letterbox(margin_height) => {
                self.context
                    .fill_rect(0.0, 0.0, self.viewport_width.into(), margin_height.into());
                self.context.fill_rect(
                    0.0,
                    (self.viewport_height as f32 - margin_height).into(),
                    self.viewport_width.into(),
                    self.viewport_height.into(),
                );
            }
            Letterbox::Pillarbox(margin_width) => {
                self.context
                    .fill_rect(0.0, 0.0, margin_width.into(), self.viewport_height.into());
                self.context.fill_rect(
                    (self.viewport_width as f32 - margin_width).into(),
                    0.0,
                    margin_width.into(),
                    self.viewport_height.into(),
                );
            }
        }
    }

    fn push_mask(&mut self) {
        // In the canvas backend, masks are implemented using two render targets.
        // We render the masker clips to the first render target.
        self.push_render_target();
    }
    fn activate_mask(&mut self) {
        // We render the maskee clips to the second render target.
        self.push_render_target();
    }
    fn pop_mask(&mut self) {
        let (maskee_canvas, maskee_context) = self.pop_render_target();
        let (masker_canvas, _masker_context) = self.pop_render_target();

        // We have to be sure to reset the transforms here so that
        // the texture is drawn starting from the upper-left corner.
        maskee_context.reset_transform().warn_on_error();
        self.context.reset_transform().warn_on_error();

        // We draw the masker onto the maskee using the "destination-in" blend mode.
        // This will filter out pixels where the maskee alpha == 0.
        maskee_context
            .set_global_composite_operation("destination-in")
            .unwrap();

        // Force alpha to 100% for the mask art, because Flash ignores alpha in masks.
        // Otherwise canvas blend modes will draw the masked clip as transparent.
        // TODO: Doesn't work on Safari because it doesn't support context.filter.
        self.color_matrix
            .set_attribute(
                "values",
                &"1.0 0 0 0 0 0 1.0 0 0 0 0 0 1.0 0 0 0 0 0 256.0 0",
            )
            .warn_on_error();

        maskee_context.set_filter("url('#_cm')");
        maskee_context
            .draw_image_with_html_canvas_element(&masker_canvas, 0.0, 0.0)
            .unwrap();
        maskee_context
            .set_global_composite_operation("source-over")
            .unwrap();
        maskee_context.set_filter("none");

        // Finally, we draw the finalized masked onto the main canvas.
        self.context.reset_transform().warn_on_error();
        self.context
            .draw_image_with_html_canvas_element(&maskee_canvas, 0.0, 0.0)
            .unwrap();
    }
}

fn swf_shape_to_svg(
    shape: &swf::Shape,
    bitmaps: &HashMap<CharacterId, (&str, u32, u32)>,
    pixelated_property_value: &str,
) -> ShapeData {
    use fnv::FnvHashSet;
    use ruffle_core::matrix::Matrix;
    use ruffle_core::shape_utils::{swf_shape_to_paths, DrawPath};
    use svg::node::element::{
        path::Data, Definitions, Image, LinearGradient, Path as SvgPath, Pattern, RadialGradient,
        Stop,
    };
    use svg::Document;
    use swf::{FillStyle, LineCapStyle, LineJoinStyle};

    // Some browsers will vomit if you try to load/draw an image with 0 width/height.
    // TODO(Herschel): Might be better to just return None in this case and skip
    // rendering altogether.
    let (width, height) = (
        f32::max(
            (shape.shape_bounds.x_max - shape.shape_bounds.x_min).to_pixels() as f32,
            1.0,
        ),
        f32::max(
            (shape.shape_bounds.y_max - shape.shape_bounds.y_min).to_pixels() as f32,
            1.0,
        ),
    );
    let mut document = Document::new()
        .set("width", width)
        .set("height", height)
        .set(
            "viewBox",
            (
                shape.shape_bounds.x_min.get(),
                shape.shape_bounds.y_min.get(),
                (shape.shape_bounds.x_max - shape.shape_bounds.x_min).get(),
                (shape.shape_bounds.y_max - shape.shape_bounds.y_min).get(),
            ),
        )
        // preserveAspectRatio must be off or Firefox will fudge with the dimensions when we draw an image onto canvas.
        .set("preserveAspectRatio", "none")
        .set("xmlns:xlink", "http://www.w3.org/1999/xlink");

    let width = (shape.shape_bounds.x_max - shape.shape_bounds.x_min).get() as f32;
    let height = (shape.shape_bounds.y_max - shape.shape_bounds.y_min).get() as f32;

    let mut bitmap_defs: FnvHashSet<CharacterId> = FnvHashSet::default();

    let mut defs = Definitions::new();
    let mut num_defs = 0;

    let mut svg_paths = vec![];
    let paths = swf_shape_to_paths(shape);
    for path in paths {
        match path {
            DrawPath::Fill { style, commands } => {
                let mut svg_path = SvgPath::new();

                svg_path = svg_path.set(
                    "fill",
                    match style {
                        FillStyle::Color(Color { r, g, b, a }) => {
                            format!("rgba({},{},{},{})", r, g, b, f32::from(*a) / 255.0)
                        }
                        FillStyle::LinearGradient(gradient) => {
                            let matrix: Matrix = Matrix::from(gradient.matrix.clone());
                            let shift = Matrix {
                                a: 32768.0 / width,
                                d: 32768.0 / height,
                                tx: swf::Twips::new(-16384),
                                ty: swf::Twips::new(-16384),
                                ..Default::default()
                            };
                            let gradient_matrix = matrix * shift;

                            let mut svg_gradient = LinearGradient::new()
                                .set("id", format!("f{}", num_defs))
                                .set("gradientUnits", "userSpaceOnUse")
                                .set(
                                    "gradientTransform",
                                    format!(
                                        "matrix({} {} {} {} {} {})",
                                        gradient_matrix.a,
                                        gradient_matrix.b,
                                        gradient_matrix.c,
                                        gradient_matrix.d,
                                        gradient_matrix.tx.get(),
                                        gradient_matrix.ty.get()
                                    ),
                                );
                            for record in &gradient.records {
                                let stop = Stop::new()
                                    .set("offset", format!("{}%", f32::from(record.ratio) / 2.55))
                                    .set(
                                        "stop-color",
                                        format!(
                                            "rgba({},{},{},{})",
                                            record.color.r,
                                            record.color.g,
                                            record.color.b,
                                            f32::from(record.color.a) / 255.0
                                        ),
                                    );
                                svg_gradient = svg_gradient.add(stop);
                            }
                            defs = defs.add(svg_gradient);

                            let fill_id = format!("url(#f{})", num_defs);
                            num_defs += 1;
                            fill_id
                        }
                        FillStyle::RadialGradient(gradient) => {
                            let matrix = Matrix::from(gradient.matrix.clone());
                            let shift = Matrix {
                                a: 32768.0,
                                d: 32768.0,
                                ..Default::default()
                            };
                            let gradient_matrix = matrix * shift;

                            let mut svg_gradient = RadialGradient::new()
                                .set("id", format!("f{}", num_defs))
                                .set("gradientUnits", "userSpaceOnUse")
                                .set("cx", "0")
                                .set("cy", "0")
                                .set("r", "0.5")
                                .set(
                                    "gradientTransform",
                                    format!(
                                        "matrix({} {} {} {} {} {})",
                                        gradient_matrix.a,
                                        gradient_matrix.b,
                                        gradient_matrix.c,
                                        gradient_matrix.d,
                                        gradient_matrix.tx.get(),
                                        gradient_matrix.ty.get()
                                    ),
                                );
                            for record in &gradient.records {
                                let stop = Stop::new()
                                    .set("offset", format!("{}%", f32::from(record.ratio) / 2.55))
                                    .set(
                                        "stop-color",
                                        format!(
                                            "rgba({},{},{},{})",
                                            record.color.r,
                                            record.color.g,
                                            record.color.b,
                                            f32::from(record.color.a) / 255.0
                                        ),
                                    );
                                svg_gradient = svg_gradient.add(stop);
                            }
                            defs = defs.add(svg_gradient);

                            let fill_id = format!("url(#f{})", num_defs);
                            num_defs += 1;
                            fill_id
                        }
                        FillStyle::FocalGradient {
                            gradient,
                            focal_point,
                        } => {
                            let matrix = Matrix::from(gradient.matrix.clone());
                            let shift = Matrix {
                                a: 32768.0,
                                d: 32768.0,
                                ..Default::default()
                            };
                            let gradient_matrix = matrix * shift;

                            let mut svg_gradient = RadialGradient::new()
                                .set("id", format!("f{}", num_defs))
                                .set("fx", focal_point / 2.0)
                                .set("gradientUnits", "userSpaceOnUse")
                                .set("cx", "0")
                                .set("cy", "0")
                                .set("r", "0.5")
                                .set(
                                    "gradientTransform",
                                    format!(
                                        "matrix({} {} {} {} {} {})",
                                        gradient_matrix.a,
                                        gradient_matrix.b,
                                        gradient_matrix.c,
                                        gradient_matrix.d,
                                        gradient_matrix.tx.get(),
                                        gradient_matrix.ty.get()
                                    ),
                                );
                            for record in &gradient.records {
                                let stop = Stop::new()
                                    .set("offset", format!("{}%", f32::from(record.ratio) / 2.55))
                                    .set(
                                        "stop-color",
                                        format!(
                                            "rgba({},{},{},{})",
                                            record.color.r,
                                            record.color.g,
                                            record.color.b,
                                            f32::from(record.color.a) / 255.0
                                        ),
                                    );
                                svg_gradient = svg_gradient.add(stop);
                            }
                            defs = defs.add(svg_gradient);

                            let fill_id = format!("url(#f{})", num_defs);
                            num_defs += 1;
                            fill_id
                        }
                        FillStyle::Bitmap {
                            id,
                            matrix,
                            is_smoothed,
                            is_repeating,
                        } => {
                            let (bitmap_data, bitmap_width, bitmap_height) =
                                bitmaps.get(&id).unwrap_or(&("", 0, 0));

                            if !bitmap_defs.contains(&id) {
                                let mut image = Image::new()
                                    .set("width", *bitmap_width)
                                    .set("height", *bitmap_height)
                                    .set("xlink:href", *bitmap_data);

                                if !*is_smoothed {
                                    image = image.set("image-rendering", pixelated_property_value);
                                }

                                let mut bitmap_pattern = Pattern::new()
                                    .set("id", format!("b{}", id))
                                    .set("patternUnits", "userSpaceOnUse");

                                if !*is_repeating {
                                    bitmap_pattern = bitmap_pattern
                                        .set("width", *bitmap_width)
                                        .set("height", *bitmap_height);
                                } else {
                                    bitmap_pattern = bitmap_pattern
                                        .set("width", *bitmap_width)
                                        .set("height", *bitmap_height)
                                        .set(
                                            "viewBox",
                                            format!("0 0 {} {}", bitmap_width, bitmap_height),
                                        );
                                }

                                bitmap_pattern = bitmap_pattern.add(image);

                                defs = defs.add(bitmap_pattern);
                                bitmap_defs.insert(*id);
                            }
                            let a = Matrix::from(matrix.clone());
                            let bitmap_matrix = a;

                            let svg_pattern = Pattern::new()
                                .set("id", format!("f{}", num_defs))
                                .set("xlink:href", format!("#b{}", id))
                                .set(
                                    "patternTransform",
                                    format!(
                                        "matrix({} {} {} {} {} {})",
                                        bitmap_matrix.a,
                                        bitmap_matrix.b,
                                        bitmap_matrix.c,
                                        bitmap_matrix.d,
                                        bitmap_matrix.tx.get(),
                                        bitmap_matrix.ty.get()
                                    ),
                                );

                            defs = defs.add(svg_pattern);

                            let fill_id = format!("url(#f{})", num_defs);
                            num_defs += 1;
                            fill_id
                        }
                    },
                );

                let mut data = Data::new();
                for command in commands {
                    data = match command {
                        DrawCommand::MoveTo { x, y } => data.move_to((x.get(), y.get())),
                        DrawCommand::LineTo { x, y } => data.line_to((x.get(), y.get())),
                        DrawCommand::CurveTo { x1, y1, x2, y2 } => {
                            data.quadratic_curve_to((x1.get(), y1.get(), x2.get(), y2.get()))
                        }
                    };
                }

                svg_path = svg_path.set("d", data);
                svg_paths.push(svg_path);
            }
            DrawPath::Stroke {
                style,
                commands,
                is_closed,
            } => {
                // Flash always renders strokes with a minimum width of 1 pixel (20 twips).
                // Additionally, many SWFs use the "hairline" stroke setting, which sets the stroke's width
                // to 1 twip. Because of the minimum, this will effectively make the stroke nearly-always render
                // as 1 pixel wide.
                // SVG doesn't have a minimum and can render strokes at fractional widths, so these hairline
                // strokes end up rendering very faintly if we use the actual width of 1 twip.
                // Therefore, we clamp the stroke width to 1 pixel (20 twips). This won't be 100% accurate
                // if the shape is scaled, but it looks much closer to the Flash Player.
                let stroke_width = std::cmp::max(style.width.get(), 20);
                let mut svg_path = SvgPath::new();
                svg_path = svg_path
                    .set("fill", "none")
                    .set(
                        "stroke",
                        format!(
                            "rgba({},{},{},{})",
                            style.color.r, style.color.g, style.color.b, style.color.a
                        ),
                    )
                    .set("stroke-width", stroke_width)
                    .set(
                        "stroke-linecap",
                        match style.start_cap {
                            LineCapStyle::Round => "round",
                            LineCapStyle::Square => "square",
                            LineCapStyle::None => "butt",
                        },
                    )
                    .set(
                        "stroke-linejoin",
                        match style.join_style {
                            LineJoinStyle::Round => "round",
                            LineJoinStyle::Bevel => "bevel",
                            LineJoinStyle::Miter(_) => "miter",
                        },
                    );

                if let LineJoinStyle::Miter(miter_limit) = style.join_style {
                    svg_path = svg_path.set("stroke-miterlimit", miter_limit);
                }

                let mut data = Data::new();
                for command in commands {
                    data = match command {
                        DrawCommand::MoveTo { x, y } => data.move_to((x.get(), y.get())),
                        DrawCommand::LineTo { x, y } => data.line_to((x.get(), y.get())),
                        DrawCommand::CurveTo { x1, y1, x2, y2 } => {
                            data.quadratic_curve_to((x1.get(), y1.get(), x2.get(), y2.get()))
                        }
                    };
                }
                if is_closed {
                    data = data.close();
                }

                svg_path = svg_path.set("d", data);
                svg_paths.push(svg_path);
            }
        }
    }

    if num_defs > 0 {
        document = document.add(defs);
    }

    for svg_path in svg_paths {
        document = document.add(svg_path);
    }

    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let svg = document.to_string();
    let svg_encoded = format!(
        "data:image/svg+xml,{}",
        utf8_percent_encode(&svg, NON_ALPHANUMERIC)
    );

    let image = HtmlImageElement::new().unwrap();
    image.set_src(&svg_encoded);

    let mut data = ShapeData(vec![]);
    data.0.push(CanvasDrawCommand::DrawImage {
        image,
        x_min: shape.shape_bounds.x_min.to_pixels(),
        y_min: shape.shape_bounds.y_min.to_pixels(),
    });

    data
}

/// Convert a series of `DrawCommands` to a `Path2d` shape.
///
/// The path can be optionally closed by setting `is_closed` to `true`.
///
/// The resulting path is in the shape's own coordinate space and needs to be
/// transformed to fit within the shape's bounds.
fn draw_commands_to_path2d(commands: &[DrawCommand], is_closed: bool) -> Path2d {
    let path = Path2d::new().unwrap();
    for command in commands {
        match command {
            DrawCommand::MoveTo { x, y } => path.move_to(x.get().into(), y.get().into()),
            DrawCommand::LineTo { x, y } => path.line_to(x.get().into(), y.get().into()),
            DrawCommand::CurveTo { x1, y1, x2, y2 } => path.quadratic_curve_to(
                x1.get().into(),
                y1.get().into(),
                x2.get().into(),
                y2.get().into(),
            ),
        };
    }

    if is_closed {
        path.close_path();
    }

    path
}

fn swf_shape_to_canvas_commands(
    shape: &swf::Shape,
    bitmaps: &HashMap<CharacterId, (&str, u32, u32)>,
    _pixelated_property_value: &str,
    context: &CanvasRenderingContext2d,
) -> Option<ShapeData> {
    use ruffle_core::matrix::Matrix;
    use ruffle_core::shape_utils::{swf_shape_to_paths, DrawPath};
    use swf::{FillStyle, LineCapStyle, LineJoinStyle};

    // Some browsers will vomit if you try to load/draw an image with 0 width/height.
    // TODO(Herschel): Might be better to just return None in this case and skip
    // rendering altogether.
    let (_width, _height) = (
        f32::max(
            (shape.shape_bounds.x_max - shape.shape_bounds.x_min).get() as f32,
            1.0,
        ),
        f32::max(
            (shape.shape_bounds.y_max - shape.shape_bounds.y_min).get() as f32,
            1.0,
        ),
    );

    let mut canvas_data = ShapeData(vec![]);

    let matrix_factory: SvgsvgElement = web_sys::window()
        .expect("window")
        .document()
        .expect("document")
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
        .expect("create_element on svg")
        .dyn_into::<SvgsvgElement>()
        .expect("an actual SVG element");

    let bounds_viewbox_matrix = matrix_factory.create_svg_matrix();
    bounds_viewbox_matrix.set_a(1.0 / 20.0);
    bounds_viewbox_matrix.set_d(1.0 / 20.0);

    let paths = swf_shape_to_paths(shape);
    for path in paths {
        match path {
            DrawPath::Fill { style, commands } => {
                let fill_style = match style {
                    FillStyle::Color(Color { r, g, b, a }) => CanvasFillStyle::Color(CanvasColor(
                        format!("rgba({},{},{},{})", r, g, b, f32::from(*a) / 255.0),
                        *r,
                        *g,
                        *b,
                        *a,
                    )),
                    FillStyle::LinearGradient(_gradient) => return None,
                    FillStyle::RadialGradient(_gradient) => return None,
                    FillStyle::FocalGradient { .. } => return None,
                    FillStyle::Bitmap {
                        id,
                        matrix,
                        is_smoothed,
                        is_repeating,
                    } => {
                        let (bitmap_data, bitmap_width, bitmap_height) =
                            bitmaps.get(&id).unwrap_or(&("", 0, 0));

                        let image = HtmlImageElement::new_with_width_and_height(
                            *bitmap_width,
                            *bitmap_height,
                        )
                        .expect("html image element");

                        if !*is_smoothed {
                            //image = image.set("image-rendering", pixelated_property_value);
                        }

                        let repeat = if !*is_repeating {
                            "no-repeat"
                        } else {
                            "repeat"
                        };

                        let bitmap_pattern = context
                            .create_pattern_with_html_image_element(&image, repeat)
                            .expect("pattern creation success")?;

                        // Set source below the pattern creation because otherwise the bitmap gets screwed up
                        // when cached? (Issue #412)
                        image.set_src(*bitmap_data);

                        let a = Matrix::from(matrix.clone());

                        let matrix = matrix_factory.create_svg_matrix();

                        matrix.set_a(a.a);
                        matrix.set_b(a.b);
                        matrix.set_c(a.c);
                        matrix.set_d(a.d);
                        matrix.set_e(a.tx.get() as f32);
                        matrix.set_f(a.ty.get() as f32);

                        bitmap_pattern.set_transform(&matrix);

                        CanvasFillStyle::Pattern(bitmap_pattern)
                    }
                };

                let path = Path2d::new().unwrap();
                path.add_path_with_transformation(
                    &draw_commands_to_path2d(&commands, false),
                    &bounds_viewbox_matrix,
                );

                canvas_data
                    .0
                    .push(CanvasDrawCommand::Fill { path, fill_style });
            }
            DrawPath::Stroke {
                style,
                commands,
                is_closed,
            } => {
                // Flash always renders strokes with a minimum width of 1 pixel (20 twips).
                // Additionally, many SWFs use the "hairline" stroke setting, which sets the stroke's width
                // to 1 twip. Because of the minimum, this will effectively make the stroke nearly-always render
                // as 1 pixel wide.
                // SVG doesn't have a minimum and can render strokes at fractional widths, so these hairline
                // strokes end up rendering very faintly if we use the actual width of 1 twip.
                // Therefore, we clamp the stroke width to 1 pixel (20 twips). This won't be 100% accurate
                // if the shape is scaled, but it looks much closer to the Flash Player.
                let line_width = std::cmp::max(style.width.get(), 20);
                let stroke_style = CanvasColor(
                    format!(
                        "rgba({},{},{},{})",
                        style.color.r, style.color.g, style.color.b, style.color.a
                    ),
                    style.color.r,
                    style.color.g,
                    style.color.b,
                    style.color.a,
                );
                let line_cap = match style.start_cap {
                    LineCapStyle::Round => "round",
                    LineCapStyle::Square => "square",
                    LineCapStyle::None => "butt",
                };
                let (line_join, miter_limit) = match style.join_style {
                    LineJoinStyle::Round => ("round", 999_999.0),
                    LineJoinStyle::Bevel => ("bevel", 999_999.0),
                    LineJoinStyle::Miter(ml) => ("miter", ml),
                };

                let path = Path2d::new().unwrap();
                path.add_path_with_transformation(
                    &draw_commands_to_path2d(&commands, is_closed),
                    &bounds_viewbox_matrix,
                );

                canvas_data.0.push(CanvasDrawCommand::Stroke {
                    path,
                    line_width: line_width as f64 / 20.0,
                    stroke_style,
                    line_cap: line_cap.to_string(),
                    line_join: line_join.to_string(),
                    miter_limit: miter_limit as f64 / 20.0,
                });
            }
        }
    }

    Some(canvas_data)
}
