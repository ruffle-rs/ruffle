use fnv::FnvHashMap;
use ruffle_core::backend::render::{
    swf, Bitmap, BitmapFormat, BitmapHandle, BitmapSource, Color, NullBitmapSource, RenderBackend,
    ShapeHandle, Transform,
};
use ruffle_core::color_transform::ColorTransform;
use ruffle_core::matrix::Matrix;
use ruffle_core::shape_utils::{DistilledShape, DrawCommand};
use ruffle_web_common::{JsError, JsResult};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    CanvasGradient, CanvasPattern, CanvasRenderingContext2d, CanvasWindingRule, DomMatrix, Element,
    HtmlCanvasElement, ImageData, Path2d,
};

const GRADIENT_TRANSFORM_THRESHOLD: f32 = 0.0001;

type Error = Box<dyn std::error::Error>;

pub struct WebCanvasRenderBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    color_matrix: Element,
    shapes: Vec<ShapeData>,
    bitmaps: FnvHashMap<BitmapHandle, BitmapData>,
    viewport_width: u32,
    viewport_height: u32,
    rect: Path2d,
    mask_state: MaskState,
    next_bitmap_handle: BitmapHandle,
}

/// Canvas-drawable shape data extracted from an SWF file.
struct ShapeData(Vec<CanvasDrawCommand>);

struct CanvasColor(String, u8, u8, u8, u8);

impl CanvasColor {
    /// Apply a color transformation to this color.
    fn color_transform(&self, cxform: &ColorTransform) -> Self {
        let Self(_, r, g, b, a) = self;
        let r = (*r as f32 * cxform.r_mult.to_f32() + (cxform.r_add as f32)) as u8;
        let g = (*g as f32 * cxform.g_mult.to_f32() + (cxform.g_add as f32)) as u8;
        let b = (*b as f32 * cxform.b_mult.to_f32() + (cxform.b_add as f32)) as u8;
        let a = (*a as f32 * cxform.a_mult.to_f32() + (cxform.a_add as f32)) as u8;
        let colstring = format!("rgba({},{},{},{})", r, g, b, f32::from(a) / 255.0);
        Self(colstring, r, g, b, a)
    }
}

/// An individual command to be drawn to the canvas.
enum CanvasDrawCommand {
    /// A command to draw a path stroke with a given style.
    Stroke {
        path: Path2d,
        line_width: f64,
        stroke_style: CanvasFillStyle,
        line_cap: String,
        line_join: String,
        miter_limit: f64,
    },

    /// A command to fill a path with a given style.
    Fill {
        path: Path2d,
        fill_style: CanvasFillStyle,
    },
}

enum CanvasFillStyle {
    Color(CanvasColor),
    Gradient(CanvasGradient),
    TransformedGradient(TransformedGradient),
    Pattern(CanvasPattern, bool),
}

struct TransformedGradient {
    gradient: CanvasGradient,
    gradient_matrix: [f64; 6],
    inverse_gradient_matrix: DomMatrix,
}

#[allow(dead_code)]
struct BitmapData {
    bitmap: Bitmap,
    image_data: ImageData,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl BitmapData {
    /// Puts the image data into a newly created <canvas>, and caches it.
    fn new(bitmap: Bitmap) -> Result<Self, Error> {
        let bitmap = bitmap.to_rgba();
        let image_data =
            ImageData::new_with_u8_clamped_array(Clamped(bitmap.data()), bitmap.width())
                .into_js_result()?;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .into_js_result()?
            .unchecked_into();
        canvas.set_width(bitmap.width());
        canvas.set_height(bitmap.height());

        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        context
            .put_image_data(&image_data, 0.0, 0.0)
            .into_js_result()?;
        Ok(BitmapData {
            bitmap,
            image_data,
            canvas,
            context,
        })
    }

    fn get_pixels(&self) -> Option<Bitmap> {
        if let Ok(bitmap_pixels) =
            self.context
                .get_image_data(0.0, 0.0, self.width().into(), self.height().into())
        {
            Some(Bitmap::new(
                self.width(),
                self.height(),
                BitmapFormat::Rgba,
                bitmap_pixels.data().to_vec(),
            ))
        } else {
            None
        }
    }

    fn width(&self) -> u32 {
        self.bitmap.width()
    }

    fn height(&self) -> u32 {
        self.bitmap.height()
    }
}

impl WebCanvasRenderBackend {
    pub fn new(
        canvas: &HtmlCanvasElement,
        is_transparent: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Request the CanvasRenderingContext2d.
        // Disable alpha for possible speedup.
        // TODO: Allow user to enable transparent background (transparent wmode in legacy Flash).
        let context_options = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &context_options,
            &"alpha".into(),
            &if is_transparent {
                wasm_bindgen::JsValue::TRUE
            } else {
                wasm_bindgen::JsValue::FALSE
            },
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
            .append_child(&color_matrix)
            .map_err(|_| "append_child failed")?;

        svg.append_child(&filter)
            .map_err(|_| "append_child failed")?;

        canvas
            .append_child(&svg)
            .map_err(|_| "append_child failed")?;

        let rect = Path2d::new().into_js_result()?;
        rect.rect(0.0, 0.0, 1.0, 1.0);

        let renderer = Self {
            canvas: canvas.clone(),
            color_matrix,
            context,
            shapes: vec![],
            bitmaps: Default::default(),
            viewport_width: 0,
            viewport_height: 0,
            rect,
            mask_state: MaskState::DrawContent,
            next_bitmap_handle: BitmapHandle(0),
        };
        Ok(renderer)
    }

    #[allow(clippy::float_cmp)]
    #[inline]
    fn set_transform(&mut self, matrix: &Matrix) {
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
        if color_transform.r_mult.is_one()
            && color_transform.g_mult.is_one()
            && color_transform.b_mult.is_one()
            && color_transform.r_add == 0
            && color_transform.g_add == 0
            && color_transform.b_add == 0
            && color_transform.a_add == 0
        {
            // Values outside the range of 0 and 1 are ignored in canvas, unlike Flash that clamps them.
            self.context
                .set_global_alpha(f64::from(color_transform.a_mult).clamp(0.0, 1.0));
        } else {
            let mult = color_transform.mult_rgba_normalized();
            let add = color_transform.add_rgba_normalized();

            let matrix_str = format!(
                "{} 0 0 0 {} 0 {} 0 0 {} 0 0 {} 0 {} 0 0 0 {} {}",
                mult[0], add[0], mult[1], add[1], mult[2], add[2], mult[3], add[3]
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

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let handle = ShapeHandle(self.shapes.len());
        let data =
            swf_shape_to_canvas_commands(&shape, bitmap_source, &self.bitmaps, &self.context);
        self.shapes.push(data);
        handle
    }

    fn replace_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
        handle: ShapeHandle,
    ) {
        let data =
            swf_shape_to_canvas_commands(&shape, bitmap_source, &self.bitmaps, &self.context);
        self.shapes[handle.0] = data;
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = ruffle_core::shape_utils::swf_glyph_to_shape(glyph);
        self.register_shape((&shape).into(), &NullBitmapSource)
    }

    fn begin_frame(&mut self, clear: Color) {
        // Reset canvas transform in case it was left in a dirty state.
        self.context.reset_transform().unwrap();

        let width = self.canvas.width();
        let height = self.canvas.height();

        if clear.a > 0 {
            let color = format!("rgba({}, {}, {}, {})", clear.r, clear.g, clear.b, clear.a);
            self.context.set_fill_style(&color.into());
            let _ = self.context.set_global_composite_operation("copy");
            self.context
                .fill_rect(0.0, 0.0, width.into(), height.into());
            let _ = self.context.set_global_composite_operation("source-over");
        } else {
            self.context
                .clear_rect(0.0, 0.0, width.into(), height.into());
        }

        self.mask_state = MaskState::DrawContent;
    }

    fn end_frame(&mut self) {
        // Noop
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform, smoothing: bool) {
        if self.mask_state == MaskState::ClearMask {
            return;
        }

        self.context.set_image_smoothing_enabled(smoothing);

        self.set_transform(&transform.matrix);
        self.set_color_filter(transform);
        if let Some(bitmap) = self.bitmaps.get(&bitmap) {
            let _ = self
                .context
                .draw_image_with_html_canvas_element(&bitmap.canvas, 0.0, 0.0);
        }
        self.clear_color_filter();
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        match &self.mask_state {
            MaskState::DrawContent => {
                self.set_transform(&transform.matrix);
                if let Some(shape) = self.shapes.get(shape.0) {
                    for command in shape.0.iter() {
                        match command {
                            CanvasDrawCommand::Fill { path, fill_style } => match fill_style {
                                CanvasFillStyle::Color(color) => {
                                    let color = color.color_transform(&transform.color_transform);
                                    self.context.set_fill_style(&color.0.into());
                                    self.context.fill_with_path_2d_and_winding(
                                        path,
                                        CanvasWindingRule::Evenodd,
                                    );
                                }
                                CanvasFillStyle::Gradient(gradient) => {
                                    self.set_color_filter(&transform);
                                    self.context.set_fill_style(gradient);
                                    self.context.fill_with_path_2d_and_winding(
                                        path,
                                        CanvasWindingRule::Evenodd,
                                    );
                                    self.clear_color_filter();
                                }
                                CanvasFillStyle::TransformedGradient(gradient) => {
                                    // Canvas has no easy way to draw gradients with an arbitrary transform,
                                    // but we can fake it by pushing the gradient's transform to the canvas,
                                    // then transforming the path itself by the inverse.
                                    self.set_color_filter(&transform);
                                    self.context.set_fill_style(&gradient.gradient);
                                    let matrix = &gradient.gradient_matrix;
                                    self.context
                                        .transform(
                                            matrix[0], matrix[1], matrix[2], matrix[3], matrix[4],
                                            matrix[5],
                                        )
                                        .warn_on_error();
                                    let untransformed_path = Path2d::new().unwrap();
                                    untransformed_path.add_path_with_transformation(
                                        path,
                                        gradient.inverse_gradient_matrix.unchecked_ref(),
                                    );
                                    self.context.fill_with_path_2d_and_winding(
                                        &untransformed_path,
                                        CanvasWindingRule::Evenodd,
                                    );
                                    self.context
                                        .set_transform(
                                            transform.matrix.a.into(),
                                            transform.matrix.b.into(),
                                            transform.matrix.c.into(),
                                            transform.matrix.d.into(),
                                            transform.matrix.tx.to_pixels(),
                                            transform.matrix.ty.to_pixels(),
                                        )
                                        .unwrap();
                                    self.clear_color_filter();
                                }
                                CanvasFillStyle::Pattern(patt, smoothed) => {
                                    self.set_color_filter(&transform);
                                    self.context.set_image_smoothing_enabled(*smoothed);
                                    self.context.set_fill_style(patt);
                                    self.context.fill_with_path_2d_and_winding(
                                        path,
                                        CanvasWindingRule::Evenodd,
                                    );
                                    self.clear_color_filter();
                                }
                            },
                            CanvasDrawCommand::Stroke {
                                path,
                                line_width,
                                stroke_style,
                                line_cap,
                                line_join,
                                miter_limit,
                            } => {
                                self.context.set_line_cap(line_cap);
                                self.context.set_line_join(line_join);
                                self.context.set_miter_limit(*miter_limit);
                                self.context.set_line_width(*line_width);
                                match stroke_style {
                                    CanvasFillStyle::Color(color) => {
                                        let color =
                                            color.color_transform(&transform.color_transform);
                                        self.context.set_stroke_style(&color.0.into());
                                        self.context.stroke_with_path(path);
                                    }
                                    CanvasFillStyle::Gradient(gradient) => {
                                        self.set_color_filter(&transform);
                                        self.context.set_stroke_style(gradient);
                                        self.context.stroke_with_path(path);
                                        self.clear_color_filter();
                                    }
                                    CanvasFillStyle::TransformedGradient(gradient) => {
                                        self.set_color_filter(&transform);
                                        self.context.set_stroke_style(&gradient.gradient);
                                        self.context.stroke_with_path(path);
                                        self.context
                                            .set_transform(
                                                transform.matrix.a.into(),
                                                transform.matrix.b.into(),
                                                transform.matrix.c.into(),
                                                transform.matrix.d.into(),
                                                transform.matrix.tx.to_pixels(),
                                                transform.matrix.ty.to_pixels(),
                                            )
                                            .unwrap();
                                        self.clear_color_filter();
                                    }
                                    CanvasFillStyle::Pattern(patt, smoothed) => {
                                        self.context.set_image_smoothing_enabled(*smoothed);
                                        self.context.set_stroke_style(patt);
                                        self.context.stroke_with_path(path);
                                        self.clear_color_filter();
                                    }
                                };
                            }
                        }
                    }
                }
            }

            // Add the shape path to the mask path.
            // Strokes are ignored.
            MaskState::DrawMask(mask_path) => {
                if let Some(shape) = self.shapes.get(shape.0) {
                    for command in shape.0.iter() {
                        if let CanvasDrawCommand::Fill { path, .. } = command {
                            mask_path.add_path_with_transformation(
                                path,
                                transform.matrix.to_dom_matrix().unchecked_ref(),
                            );
                        }
                    }
                }
            }

            // Canvas backend doesn't have to do anything to clear masks.
            MaskState::ClearMask => (),
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: &Matrix) {
        match &self.mask_state {
            MaskState::DrawContent => {
                self.set_transform(matrix);
                self.clear_color_filter();
                self.context.set_fill_style(
                    &format!(
                        "rgba({},{},{},{})",
                        color.r,
                        color.g,
                        color.b,
                        f32::from(color.a) / 255.0
                    )
                    .into(),
                );
                self.context.fill_rect(0.0, 0.0, 1.0, 1.0);
            }
            MaskState::DrawMask(mask_path) => {
                mask_path.add_path_with_transformation(
                    &self.rect,
                    matrix.to_dom_matrix().unchecked_ref(),
                );
            }
            MaskState::ClearMask => (),
        }
    }

    fn push_mask(&mut self) {
        if self.mask_state == MaskState::DrawContent {
            // Save the current mask layer so that it can be restored when the mask is popped.
            self.context.save();
            self.mask_state = MaskState::DrawMask(Path2d::new().unwrap());
        }
    }

    fn activate_mask(&mut self) {
        if let MaskState::DrawMask(mask_path) = &self.mask_state {
            self.context.reset_transform().unwrap();
            // Apply the clipping path to the canvas for future draws.
            // TODO: Canvas almost, but not completely, provides a clean way to implement Flash masks.
            // Subsequent calls to `CanvasRenderingContext2d.clip` support nested masks (intersection),
            // but Flash art is sometimes in layers where one path is filled on top of another (union).
            // We use non-zero winding here to handle this case, but this will be incorrect in the
            // other cases such as self-intersecting shapes. The Flash IDE actively avoids exporting
            // shapes and masks for both of these cases, so it shouldn't be common.
            // Most likely it would happen with dynamiuc masks via drawing API or similar.
            // A possible improvement is to choose the winding rule based on whether the shape has
            // layers or not (via a flag in DistilledShape?)
            self.context
                .clip_with_path_2d_and_winding(&mask_path, CanvasWindingRule::Nonzero);
            self.mask_state = MaskState::DrawContent;
        }
    }

    fn deactivate_mask(&mut self) {
        if self.mask_state == MaskState::DrawContent {
            self.mask_state = MaskState::ClearMask;
        }
    }

    fn pop_mask(&mut self) {
        if self.mask_state == MaskState::ClearMask {
            // Pop the previous clipping state.
            self.context.restore();
            self.mask_state = MaskState::DrawContent;
        }
    }

    fn get_bitmap_pixels(&mut self, bitmap: BitmapHandle) -> Option<Bitmap> {
        let bitmap = &self.bitmaps[&bitmap];
        bitmap.get_pixels()
    }

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, Error> {
        let handle = self.next_bitmap_handle;
        self.next_bitmap_handle = BitmapHandle(self.next_bitmap_handle.0 + 1);
        let bitmap_data = BitmapData::new(bitmap)?;
        self.bitmaps.insert(handle, bitmap_data);
        Ok(handle)
    }

    fn unregister_bitmap(&mut self, bitmap: BitmapHandle) -> Result<(), Error> {
        self.bitmaps.remove(&bitmap);
        Ok(())
    }

    fn update_texture(
        &mut self,
        handle: BitmapHandle,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<BitmapHandle, Error> {
        // TODO: Could be optimized to a single put_image_data call
        // in case it is already stored as a canvas+context.
        self.bitmaps.insert(
            handle,
            BitmapData::new(Bitmap::new(width, height, BitmapFormat::Rgba, rgba))?,
        );
        Ok(handle)
    }
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
    shape: &DistilledShape,
    bitmap_source: &dyn BitmapSource,
    bitmaps: &FnvHashMap<BitmapHandle, BitmapData>,
    context: &CanvasRenderingContext2d,
) -> ShapeData {
    use ruffle_core::shape_utils::DrawPath;
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

    let bounds_viewbox_matrix = DomMatrix::new().unwrap();
    bounds_viewbox_matrix.set_a(1.0 / 20.0);
    bounds_viewbox_matrix.set_d(1.0 / 20.0);

    for path in &shape.paths {
        let (style, commands, is_fill, is_closed) = match &path {
            DrawPath::Fill {
                style, commands, ..
            } => (*style, commands, true, false),
            DrawPath::Stroke {
                style,
                commands,
                is_closed,
            } => (style.fill_style(), commands, false, *is_closed),
        };
        let fill_style = match style {
            FillStyle::Color(Color { r, g, b, a }) => CanvasFillStyle::Color(CanvasColor(
                format!("rgba({},{},{},{})", r, g, b, f32::from(*a) / 255.0),
                *r,
                *g,
                *b,
                *a,
            )),
            FillStyle::LinearGradient(gradient) => {
                create_linear_gradient(context, gradient, is_fill).unwrap()
            }
            FillStyle::RadialGradient(gradient) => {
                create_radial_gradient(context, gradient, 0.0, is_fill).unwrap()
            }
            FillStyle::FocalGradient {
                gradient,
                focal_point,
            } => create_radial_gradient(context, gradient, focal_point.to_f64(), is_fill).unwrap(),
            FillStyle::Bitmap {
                id,
                matrix,
                is_smoothed,
                is_repeating,
            } => {
                if let Some(bitmap) = bitmap_source
                    .bitmap(*id)
                    .and_then(|bitmap| bitmaps.get(&bitmap.handle))
                {
                    let repeat = if !*is_repeating {
                        // NOTE: The WebGL backend does clamping in this case, just like
                        // Flash Player, but CanvasPattern has no such option...
                        "no-repeat"
                    } else {
                        "repeat"
                    };

                    let bitmap_pattern = if let Ok(Some(bitmap_pattern)) =
                        context.create_pattern_with_html_canvas_element(&bitmap.canvas, repeat)
                    {
                        bitmap_pattern
                    } else {
                        log::warn!("Unable to create bitmap pattern for bitmap ID {}", id);
                        continue;
                    };

                    bitmap_pattern.set_transform(matrix.to_dom_matrix().unchecked_ref());

                    CanvasFillStyle::Pattern(bitmap_pattern, *is_smoothed)
                } else {
                    log::error!("Couldn't fill shape with unknown bitmap {}", id);
                    CanvasFillStyle::Color(CanvasColor("rgba(0,0,0,0)".to_string(), 0, 0, 0, 0))
                }
            }
        };

        let canvas_path = Path2d::new().unwrap();
        canvas_path.add_path_with_transformation(
            &draw_commands_to_path2d(commands, is_closed),
            bounds_viewbox_matrix.unchecked_ref(),
        );

        match path {
            DrawPath::Fill { .. } => {
                canvas_data.0.push(CanvasDrawCommand::Fill {
                    path: canvas_path,
                    fill_style,
                });
            }
            DrawPath::Stroke { style, .. } => {
                // Flash always renders strokes with a minimum width of 1 pixel (20 twips).
                // Additionally, many SWFs use the "hairline" stroke setting, which sets the stroke's width
                // to 1 twip. Because of the minimum, this will effectively make the stroke nearly-always render
                // as 1 pixel wide.
                // SVG doesn't have a minimum and can render strokes at fractional widths, so these hairline
                // strokes end up rendering very faintly if we use the actual width of 1 twip.
                // Therefore, we clamp the stroke width to 1 pixel (20 twips). This won't be 100% accurate
                // if the shape is scaled, but it looks much closer to the Flash Player.
                let line_width = std::cmp::max(style.width().get(), 20);
                let line_cap = match style.start_cap() {
                    LineCapStyle::Round => "round",
                    LineCapStyle::Square => "square",
                    LineCapStyle::None => "butt",
                };
                let (line_join, miter_limit) = match style.join_style() {
                    LineJoinStyle::Round => ("round", 999_999.0),
                    LineJoinStyle::Bevel => ("bevel", 999_999.0),
                    LineJoinStyle::Miter(ml) => ("miter", ml.to_f32()),
                };
                canvas_data.0.push(CanvasDrawCommand::Stroke {
                    path: canvas_path,
                    line_width: line_width as f64 / 20.0,
                    stroke_style: fill_style,
                    line_cap: line_cap.to_string(),
                    line_join: line_join.to_string(),
                    miter_limit: miter_limit as f64 / 20.0,
                });
            }
        }
    }
    canvas_data
}

fn create_linear_gradient(
    context: &CanvasRenderingContext2d,
    gradient: &swf::Gradient,
    is_fill: bool,
) -> Result<CanvasFillStyle, JsError> {
    // Canvas linear gradients are configured via the line endpoints, so we only need
    // to transform it if the basis is not orthogonal (skew in the transform).
    let transformed = if is_fill {
        let dot = gradient.matrix.a * gradient.matrix.c + gradient.matrix.b * gradient.matrix.d;
        dot.to_f32().abs() > GRADIENT_TRANSFORM_THRESHOLD
    } else {
        // TODO: Gradient transforms don't work correctly with strokes.
        false
    };
    let create_fn = |matrix: swf::Matrix, gradient_scale: f64| {
        let start = matrix * (swf::Twips::new(-16384), swf::Twips::ZERO);
        let end = matrix * (swf::Twips::new(16384), swf::Twips::ZERO);
        // If we have to scale the gradient due to spread mode, scale the endpoints away from the center.
        let dx = 0.5 * (gradient_scale - 1.0) * (end.0 - start.0).to_pixels();
        let dy = 0.5 * (gradient_scale - 1.0) * (end.1 - start.1).to_pixels();
        Ok(context.create_linear_gradient(
            start.0.to_pixels() - dx,
            start.1.to_pixels() - dy,
            end.0.to_pixels() + dx,
            end.1.to_pixels() + dy,
        ))
    };
    swf_to_canvas_gradient(gradient, transformed, create_fn)
}

fn create_radial_gradient(
    context: &CanvasRenderingContext2d,
    gradient: &swf::Gradient,
    focal_point: f64,
    is_fill: bool,
) -> Result<CanvasFillStyle, JsError> {
    // Canvas radial gradients can not be elliptical or skewed, so transform if there
    // is a non-uniform scale or skew.
    // A scale rotation matrix is always of the form:
    // [[a  b]
    //  [-b a]]
    let transformed = if is_fill {
        (gradient.matrix.a - gradient.matrix.d).to_f32().abs() > GRADIENT_TRANSFORM_THRESHOLD
            || (gradient.matrix.b + gradient.matrix.c).to_f32().abs() > GRADIENT_TRANSFORM_THRESHOLD
    } else {
        // TODO: Gradient transforms don't work correctly with strokes.
        false
    };
    let create_fn = |matrix: swf::Matrix, gradient_scale: f64| {
        let focal_center = matrix
            * (
                swf::Twips::new((focal_point * 16384.0) as i32),
                swf::Twips::ZERO,
            );
        let center = matrix * (swf::Twips::ZERO, swf::Twips::ZERO);
        let end = matrix * (swf::Twips::new(16384), swf::Twips::ZERO);
        let dx = (end.0 - center.0).to_pixels();
        let dy = (end.1 - center.1).to_pixels();
        let radius = (dx * dx + dy * dy).sqrt();
        context
            .create_radial_gradient(
                focal_center.0.to_pixels(),
                focal_center.1.to_pixels(),
                0.0,
                center.0.to_pixels(),
                center.1.to_pixels(),
                // Radius needs to be scaled if gradient spread mode is active.
                radius * gradient_scale,
            )
            .into_js_result()
    };
    swf_to_canvas_gradient(gradient, transformed, create_fn)
}

/// Converts an SWF gradient to a canvas gradient.
///
/// If the SWF gradient has a "simple" transform, this is a direct translation to `CanvasGradient`.
/// If transform is "complex" (skewing or non-uniform scaling), we have to do some trickery and
/// transform the entire path, because canvas does not have a direct way to render a transformed
/// gradient.
fn swf_to_canvas_gradient(
    swf_gradient: &swf::Gradient,
    transformed: bool,
    mut create_gradient_fn: impl FnMut(swf::Matrix, f64) -> Result<CanvasGradient, JsError>,
) -> Result<CanvasFillStyle, JsError> {
    let matrix = if transformed {
        // When we are rendering a complex gradient, the gradient transform is handled later by
        // transforming the path before rendering; so use the indentity matrix here.
        swf::Matrix::scale(swf::Fixed16::from_f64(20.0), swf::Fixed16::from_f64(20.0))
    } else {
        swf_gradient.matrix
    };

    const NUM_REPEATS: f32 = 25.0;
    let gradient_scale = if swf_gradient.spread == swf::GradientSpread::Pad {
        1.0
    } else {
        f64::from(NUM_REPEATS)
    };

    // Canvas does not have support for spread/repeat modes (reflect+repeat), so we have to
    // simulate these repeat modes by duplicating color stops.
    // TODO: We'll hit the edge if the gradient is shrunk way down, but don't think we can do
    // anything better using the current Canvas API. Maybe we could consider the size of the
    // shape here to make sure we fill the area.
    let canvas_gradient = create_gradient_fn(matrix, gradient_scale)?;
    let color_stops: Vec<_> = swf_gradient
        .records
        .iter()
        .map(|record| {
            (
                f32::from(record.ratio) / 255.0,
                format!(
                    "rgba({},{},{},{})",
                    record.color.r,
                    record.color.g,
                    record.color.b,
                    f32::from(record.color.a) / 255.0
                ),
            )
        })
        .collect();

    match swf_gradient.spread {
        swf::GradientSpread::Pad => {
            for stop in color_stops {
                canvas_gradient
                    .add_color_stop(stop.0, &stop.1)
                    .warn_on_error();
            }
        }
        swf::GradientSpread::Reflect => {
            let mut t = 0.0;
            let step = 1.0 / NUM_REPEATS;
            while t < 1.0 {
                // Add the colors forward.
                for stop in &color_stops {
                    canvas_gradient
                        .add_color_stop(t + stop.0 * step, &stop.1)
                        .warn_on_error();
                }
                t += step;
                // Add the colors backward.
                for stop in color_stops.iter().rev() {
                    canvas_gradient
                        .add_color_stop(t + (1.0 - stop.0) * step, &stop.1)
                        .warn_on_error();
                }
                t += step;
            }
        }
        swf::GradientSpread::Repeat => {
            let first_stop = color_stops.first().unwrap();
            let last_stop = color_stops.last().unwrap();
            let mut t = 0.0;
            let step = 1.0 / NUM_REPEATS;
            while t < 1.0 {
                // Duplicate the start/end stops to ensure we don't blend between the seams.
                canvas_gradient
                    .add_color_stop(t, &first_stop.1)
                    .warn_on_error();
                for stop in &color_stops {
                    canvas_gradient
                        .add_color_stop(t + stop.0 * step, &stop.1)
                        .warn_on_error();
                }
                canvas_gradient
                    .add_color_stop(t + step, &last_stop.1)
                    .warn_on_error();
                t += step;
            }
        }
    }

    if transformed {
        // When we render this gradient, we will push the gradient's transform to the canvas,
        // and then transform the path itself by the inverse.
        let matrix = swf_gradient.matrix.to_dom_matrix();
        let inverse_gradient_matrix = matrix.inverse();
        Ok(CanvasFillStyle::TransformedGradient(TransformedGradient {
            gradient: canvas_gradient,
            gradient_matrix: [
                matrix.a(),
                matrix.b(),
                matrix.c(),
                matrix.d(),
                matrix.e(),
                matrix.f(),
            ],
            inverse_gradient_matrix,
        }))
    } else {
        Ok(CanvasFillStyle::Gradient(canvas_gradient))
    }
}

/// The current masking behavior of the canvas.
#[derive(Debug, Eq, PartialEq)]
enum MaskState {
    // Content is being drawn.
    DrawContent,

    // A clipping layer is being drawn.
    // Paths should be added to the `Path2d` instead of drawn to the canvas.
    DrawMask(Path2d),

    // A clipping layer is being cleared.
    // On a canvas, all draws are a no-op until the clear is complete.
    ClearMask,
}

/// Extension trait for easily converting from Ruffle matrices to `DomMatrix`.
trait MatrixExt {
    fn to_dom_matrix(&self) -> DomMatrix;
}

impl MatrixExt for Matrix {
    fn to_dom_matrix(&self) -> DomMatrix {
        DomMatrix::new_with_array64(
            [
                self.a.into(),
                self.b.into(),
                self.c.into(),
                self.d.into(),
                self.tx.to_pixels(),
                self.ty.to_pixels(),
            ]
            .as_mut_slice(),
        )
        .unwrap()
    }
}

impl MatrixExt for swf::Matrix {
    fn to_dom_matrix(&self) -> DomMatrix {
        DomMatrix::new_with_array64(
            [
                self.a.to_f64() / 20.0,
                self.b.to_f64() / 20.0,
                self.c.to_f64() / 20.0,
                self.d.to_f64() / 20.0,
                self.tx.to_pixels(),
                self.ty.to_pixels(),
            ]
            .as_mut_slice(),
        )
        .unwrap()
    }
}
