#![deny(clippy::unwrap_used)]
// Remove this when we start using `Rc` when compiling for wasm
#![allow(clippy::arc_with_non_send_sync)]

use ruffle_render::backend::{
    BitmapCacheEntry, Context3D, Context3DProfile, PixelBenderOutput, PixelBenderTarget,
    RenderBackend, ShapeHandle, ShapeHandleImpl, ViewportDimensions,
};
use ruffle_render::bitmap::{
    Bitmap, BitmapHandle, BitmapHandleImpl, BitmapSource, PixelRegion, PixelSnapping, RgbaBufRead,
    SyncHandle,
};
use ruffle_render::commands::{CommandHandler, CommandList, RenderBlendMode};
use ruffle_render::error::Error;
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::shape_utils::{DistilledShape, DrawCommand, LineScaleMode, LineScales};
use ruffle_render::transform::Transform;
use ruffle_web_common::{JsError, JsResult};
use std::borrow::Cow;
use std::sync::Arc;
use swf::{BlendMode, Color, ColorTransform, Point, Twips};
use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{
    CanvasGradient, CanvasPattern, CanvasRenderingContext2d, CanvasWindingRule, DomMatrix, Element,
    HtmlCanvasElement, ImageData, Path2d,
};

const GRADIENT_TRANSFORM_THRESHOLD: f32 = 0.0001;

pub struct WebCanvasRenderBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    color_matrix: Element,
    viewport_width: u32,
    viewport_height: u32,
    rect: Path2d,
    line: Path2d,
    line_rect: Path2d,
    mask_state: MaskState,
    blend_modes: Vec<RenderBlendMode>,

    // This is currently unused - we just store it to report
    // in `get_viewport_dimensions`
    viewport_scale_factor: f64,
}

/// Canvas-drawable shape data extracted from an SWF file.
#[derive(Debug)]
struct ShapeData(Vec<CanvasDrawCommand>);

impl ShapeHandleImpl for ShapeData {}

fn as_shape_data(handle: &ShapeHandle) -> &ShapeData {
    <dyn ShapeHandleImpl>::downcast_ref(&*handle.0)
        .expect("Shape handle must be a Canvas ShapeData")
}

#[derive(Debug)]
struct CanvasColor(Color, String);

impl From<Color> for CanvasColor {
    fn from(color: Color) -> Self {
        Self(
            color,
            format!(
                "rgba({},{},{},{})",
                color.r,
                color.g,
                color.b,
                f32::from(color.a) / 255.0
            ),
        )
    }
}

impl CanvasColor {
    /// Apply a color transformation to this color.
    fn color_transform(&self, cxform: &ColorTransform) -> Self {
        (cxform * self.0).into()
    }
}

/// An individual command to be drawn to the canvas.
#[derive(Debug)]
enum CanvasDrawCommand {
    /// A command to draw a path stroke with a given style.
    Stroke {
        path: Path2d,
        line_width: f64,
        stroke_style: CanvasStrokeStyle,
        line_cap: String,
        line_join: String,
        miter_limit: f64,
        scale_mode: LineScaleMode,
    },

    /// A command to fill a path with a given style.
    Fill {
        path: Path2d,
        fill_style: CanvasFillStyle,
    },
}

/// Fill style for a canvas path.
#[derive(Debug)]
enum CanvasFillStyle {
    Color(CanvasColor),
    Gradient(Gradient),
    Bitmap(CanvasBitmap),
}

#[derive(Debug)]
struct Gradient {
    gradient: CanvasGradient,
    transform: Option<GradientTransform>,
}

/// A "complex" gradient transform, such as elliptical or skewed gradients.
///
/// Canvas does not provide an API for arbitrary gradient transforms, so we cheat by applying the
/// inverse of the gradient transform to the path, and then drawing the path with the gradient transform.
/// This results in an non-transformed shape with a transformed gradient.
#[derive(Debug)]
struct GradientTransform {
    matrix: [f64; 6],
    inverse_matrix: DomMatrix,
}

/// Stroke style for a canvas path.
///
/// Gradients are handled differently for strokes vs. fills.
#[derive(Debug)]
enum CanvasStrokeStyle {
    Color(CanvasColor),
    Gradient(swf::Gradient, Option<f64>),
    Bitmap(CanvasBitmap),
}

#[derive(Debug)]
struct CanvasBitmap {
    pattern: CanvasPattern,
    matrix: Matrix,
    smoothed: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
struct BitmapData {
    image_data: ImageData,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl BitmapHandleImpl for BitmapData {}

fn as_bitmap_data(handle: &BitmapHandle) -> &BitmapData {
    <dyn BitmapHandleImpl>::downcast_ref(&*handle.0)
        .expect("Bitmap handle must be a Canvas BitmapData")
}

impl BitmapData {
    /// Puts the image data into a newly created <canvas>, and caches it.
    fn with_bitmap(bitmap: Bitmap) -> Result<Self, JsValue> {
        let bitmap = bitmap.to_rgba();
        let image_data =
            ImageData::new_with_u8_clamped_array(Clamped(bitmap.data()), bitmap.width())
                .into_js_result()?;
        Self::with_image_data(image_data)
    }

    fn empty(width: u32, height: u32) -> Result<Self, JsValue> {
        let image_data = ImageData::new_with_sw(width, height).into_js_result()?;
        Self::with_image_data(image_data)
    }

    fn with_image_data(image_data: ImageData) -> Result<Self, JsValue> {
        let window = web_sys::window().expect("window()");
        let document = window.document().expect("document()");
        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .into_js_result()?
            .unchecked_into();
        canvas.set_width(image_data.width());
        canvas.set_height(image_data.height());

        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .expect("get_context method must return a value")
            .dyn_into()
            .expect("get_context method returned something other than a CanvasRenderingContext2d");
        context
            .put_image_data(&image_data, 0.0, 0.0)
            .into_js_result()?;
        Ok(BitmapData {
            image_data,
            canvas,
            context,
        })
    }

    fn update_pixels(&self, bitmap: Bitmap) -> Result<(), JsValue> {
        let bitmap = bitmap.to_rgba();
        let image_data =
            ImageData::new_with_u8_clamped_array(Clamped(bitmap.data()), bitmap.width())
                .into_js_result()?;
        self.canvas.set_width(bitmap.width());
        self.canvas.set_height(bitmap.height());
        self.context
            .put_image_data(&image_data, 0.0, 0.0)
            .into_js_result()?;
        Ok(())
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
            &JsValue::from_bool(is_transparent),
        );
        let context: CanvasRenderingContext2d = canvas
            .get_context_with_context_options("2d", &context_options)
            .into_js_result()?
            .ok_or("Could not create context")?
            .dyn_into()
            .map_err(|_| "Expected CanvasRenderingContext2d")?;

        let window = web_sys::window().expect("window()");
        let document = window.document().expect("document()");

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

        let line = Path2d::new().into_js_result()?;
        line.move_to(0.0, 0.0);
        line.line_to(1.0, 0.0);

        let line_rect = Path2d::new().into_js_result()?;
        line_rect.move_to(0.0, 0.0);
        line_rect.line_to(1.0, 0.0);
        line_rect.line_to(1.0, 1.0);
        line_rect.line_to(0.0, 1.0);
        line_rect.close_path();

        let renderer = Self {
            canvas: canvas.clone(),
            color_matrix,
            context,
            viewport_width: 0,
            viewport_height: 0,
            viewport_scale_factor: 1.0,
            rect,
            line,
            line_rect,
            mask_state: MaskState::DrawContent,
            blend_modes: vec![RenderBlendMode::Builtin(BlendMode::Normal)],
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
            .warn_on_error();
    }

    #[allow(clippy::float_cmp)]
    #[inline]
    fn set_color_filter(&self, transform: &Transform) {
        let color_transform = &transform.color_transform;
        if color_transform.r_multiply.is_one()
            && color_transform.g_multiply.is_one()
            && color_transform.b_multiply.is_one()
            && color_transform.r_add == 0
            && color_transform.g_add == 0
            && color_transform.b_add == 0
            && color_transform.a_add == 0
        {
            // Values outside the range of 0 and 1 are ignored in canvas, unlike Flash that clamps them.
            self.context
                .set_global_alpha(f64::from(color_transform.a_multiply).clamp(0.0, 1.0));
        } else {
            let mult = color_transform.mult_rgba_normalized();
            let add = color_transform.add_rgba_normalized();

            let matrix_str = format!(
                "{} 0 0 0 {} 0 {} 0 0 {} 0 0 {} 0 {} 0 0 0 {} {}",
                mult[0], add[0], mult[1], add[1], mult[2], add[2], mult[3], add[3]
            );

            self.color_matrix
                .set_attribute("values", &matrix_str)
                .warn_on_error();

            self.context.set_filter("url('#_cm')");
        }
    }

    #[inline]
    fn clear_color_filter(&self) {
        self.context.set_filter("none");
        self.context.set_global_alpha(1.0);
    }

    #[inline]
    fn set_line_style(&self, color: Color) {
        self.context.set_line_cap("butt");
        self.context.set_line_width(1.0);
        self.context.set_line_join("miter");
        self.context.set_stroke_style(
            &format!(
                "rgba({},{},{},{})",
                color.r,
                color.g,
                color.b,
                f32::from(color.a) / 255.0,
            )
            .into(),
        );
    }

    fn apply_blend_mode(&mut self, blend: RenderBlendMode) {
        // TODO: Objects with a blend mode need to be rendered to an intermediate buffer first,
        // but for now we render each child directly to the canvas. This should look reasonable for most
        // common cases.
        // While canvas has built in support for most of the blend modes, a few aren't supported.
        let mode = match blend {
            RenderBlendMode::Builtin(BlendMode::Normal) => "source-over",
            RenderBlendMode::Builtin(BlendMode::Layer) => "source-over", // Requires intermediate buffer.
            RenderBlendMode::Builtin(BlendMode::Multiply) => "multiply",
            RenderBlendMode::Builtin(BlendMode::Screen) => "screen",
            RenderBlendMode::Builtin(BlendMode::Lighten) => "lighten",
            RenderBlendMode::Builtin(BlendMode::Darken) => "darken",
            RenderBlendMode::Builtin(BlendMode::Difference) => "difference",
            RenderBlendMode::Builtin(BlendMode::Add) => "lighter",
            RenderBlendMode::Builtin(BlendMode::Subtract) => "difference", // Not exposed by canvas, rendered as difference.
            RenderBlendMode::Builtin(BlendMode::Invert) => "source-over",  // Not exposed by canvas.
            RenderBlendMode::Builtin(BlendMode::Alpha) => "source-over", // Requires intermediate buffer.
            RenderBlendMode::Builtin(BlendMode::Erase) => "source-over", // Requires intermediate buffer.
            RenderBlendMode::Builtin(BlendMode::Overlay) => "overlay",
            RenderBlendMode::Builtin(BlendMode::HardLight) => "hard-light",
            RenderBlendMode::Shader(_) => "source-over", // Canvas does not support shaders
        };
        self.context
            .set_global_composite_operation(mode)
            .expect("Failed to update BlendMode");
    }

    fn begin_frame(&mut self, clear: Color) {
        // Reset canvas transform in case it was left in a dirty state.
        self.context.reset_transform().warn_on_error();

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

    fn push_blend_mode(&mut self, blend: RenderBlendMode) {
        if !same_blend_mode(self.blend_modes.last(), &blend) {
            self.apply_blend_mode(blend.clone());
        }
        self.blend_modes.push(blend);
    }

    fn pop_blend_mode(&mut self) {
        let old = self.blend_modes.pop();
        // We should never pop our base 'BlendMode::Normal'
        let current = self
            .blend_modes
            .last()
            .unwrap_or(&RenderBlendMode::Builtin(BlendMode::Normal));
        if !same_blend_mode(old.as_ref(), current) {
            self.apply_blend_mode(current.clone());
        }
    }

    fn draw_lines(&mut self, color: Color, mut matrix: Matrix, rect: bool) {
        matrix.tx += Twips::HALF;
        matrix.ty += Twips::HALF;
        let dom_matrix = matrix.to_dom_matrix();
        let stroke = if rect { &self.line_rect } else { &self.line };
        match &self.mask_state {
            MaskState::DrawContent => {
                self.clear_color_filter();

                // The transform needs to be applied to the path directly,
                // otherwise its thickness will also be transformed.
                let _ = self.context.reset_transform();
                let transformed_stroke = Path2d::new().expect("new Path2d");
                transformed_stroke.add_path_with_transformation(stroke, dom_matrix.unchecked_ref());

                self.set_line_style(color);
                self.context.stroke_with_path(&transformed_stroke);
            }
            MaskState::DrawMask(mask_path) => {
                mask_path.add_path_with_transformation(stroke, dom_matrix.unchecked_ref());
            }
            MaskState::ClearMask => (),
        }
    }
}

impl RenderBackend for WebCanvasRenderBackend {
    fn set_viewport_dimensions(&mut self, dimensions: ViewportDimensions) {
        self.viewport_width = dimensions.width;
        self.viewport_height = dimensions.height;
        self.viewport_scale_factor = dimensions.scale_factor;
    }

    fn viewport_dimensions(&self) -> ViewportDimensions {
        ViewportDimensions {
            width: self.viewport_width,
            height: self.viewport_height,
            scale_factor: self.viewport_scale_factor,
        }
    }

    fn register_shape(
        &mut self,
        shape: DistilledShape,
        bitmap_source: &dyn BitmapSource,
    ) -> ShapeHandle {
        let data = swf_shape_to_canvas_commands(&shape, bitmap_source, self);
        ShapeHandle(Arc::new(ShapeData(data)))
    }

    fn render_offscreen(
        &mut self,
        _handle: BitmapHandle,
        _commands: CommandList,
        _quality: StageQuality,
        _bounds: PixelRegion,
    ) -> Option<Box<dyn SyncHandle>> {
        None
    }

    fn submit_frame(
        &mut self,
        clear: Color,
        commands: CommandList,
        cache_entries: Vec<BitmapCacheEntry>,
    ) {
        if !cache_entries.is_empty() {
            panic!("Bitmap caching is unavailable on the canvas backend");
        }
        self.begin_frame(clear);
        commands.execute(self);
    }

    fn register_bitmap(&mut self, bitmap: Bitmap) -> Result<BitmapHandle, Error> {
        let bitmap_data = BitmapData::with_bitmap(bitmap).map_err(Error::JavascriptError)?;
        Ok(BitmapHandle(Arc::new(bitmap_data)))
    }

    fn update_texture(
        &mut self,
        handle: &BitmapHandle,
        bitmap: Bitmap,
        _region: PixelRegion,
    ) -> Result<(), Error> {
        let data = as_bitmap_data(handle);
        data.update_pixels(bitmap).map_err(Error::JavascriptError)?;
        Ok(())
    }

    fn create_context3d(
        &mut self,
        _profile: Context3DProfile,
    ) -> Result<Box<dyn Context3D>, Error> {
        Err(Error::Unimplemented("createContext3D".into()))
    }
    fn context3d_present(&mut self, _context: &mut dyn Context3D) -> Result<(), Error> {
        Err(Error::Unimplemented("Context3D.present".into()))
    }

    fn debug_info(&self) -> Cow<'static, str> {
        Cow::Borrowed("Renderer: Canvas")
    }

    fn name(&self) -> &'static str {
        "canvas"
    }

    fn set_quality(&mut self, _quality: StageQuality) {}

    fn compile_pixelbender_shader(
        &mut self,
        _shader: ruffle_render::pixel_bender::PixelBenderShader,
    ) -> Result<ruffle_render::pixel_bender::PixelBenderShaderHandle, Error> {
        Err(Error::Unimplemented("compile_pixelbender_shader".into()))
    }

    fn run_pixelbender_shader(
        &mut self,
        _handle: ruffle_render::pixel_bender::PixelBenderShaderHandle,
        _arguments: &[ruffle_render::pixel_bender::PixelBenderShaderArgument],
        _target: &PixelBenderTarget,
    ) -> Result<PixelBenderOutput, Error> {
        Err(Error::Unimplemented("run_pixelbender_shader".into()))
    }

    fn resolve_sync_handle(
        &mut self,
        _handle: Box<dyn SyncHandle>,
        _with_rgba: RgbaBufRead,
    ) -> Result<(), Error> {
        Err(Error::Unimplemented("Sync handle resolution".into()))
    }

    fn create_empty_texture(&mut self, width: u32, height: u32) -> Result<BitmapHandle, Error> {
        let bitmap_data = BitmapData::empty(width, height).map_err(Error::JavascriptError)?;
        Ok(BitmapHandle(Arc::new(bitmap_data)))
    }
}

impl CommandHandler for WebCanvasRenderBackend {
    fn render_bitmap(
        &mut self,
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        _pixel_snapping: PixelSnapping,
    ) {
        if self.mask_state == MaskState::ClearMask {
            return;
        }

        self.context.set_image_smoothing_enabled(smoothing);

        self.set_transform(&transform.matrix);
        self.set_color_filter(&transform);
        let bitmap = as_bitmap_data(&bitmap);
        let _ = self
            .context
            .draw_image_with_html_canvas_element(&bitmap.canvas, 0.0, 0.0);
        self.clear_color_filter();
    }

    fn render_stage3d(&mut self, _bitmap: BitmapHandle, _transform: Transform) {
        panic!("Stage3D should not have been created on canvas backend")
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
        let shape = as_shape_data(&shape);
        match &self.mask_state {
            MaskState::DrawContent => {
                let mut line_scale = LineScales::new(&transform.matrix);
                let dom_matrix = transform.matrix.to_dom_matrix();
                let mut transform_dirty = true;
                for command in shape.0.iter() {
                    match command {
                        CanvasDrawCommand::Fill { path, fill_style } => {
                            if transform_dirty {
                                let _ = self.context.set_transform(
                                    transform.matrix.a.into(),
                                    transform.matrix.b.into(),
                                    transform.matrix.c.into(),
                                    transform.matrix.d.into(),
                                    transform.matrix.tx.to_pixels(),
                                    transform.matrix.ty.to_pixels(),
                                );
                                transform_dirty = false;
                            }
                            match fill_style {
                                CanvasFillStyle::Color(color) => {
                                    let color = color.color_transform(&transform.color_transform);
                                    self.context.set_fill_style(&color.1.into());
                                    self.context.fill_with_path_2d_and_winding(
                                        path,
                                        CanvasWindingRule::Evenodd,
                                    );
                                }
                                CanvasFillStyle::Gradient(gradient) => {
                                    self.set_color_filter(&transform);
                                    self.context.set_fill_style(&gradient.gradient);

                                    if let Some(gradient_transform) = &gradient.transform {
                                        // Canvas has no easy way to draw gradients with an arbitrary transform,
                                        // but we can fake it by pushing the gradient's transform to the canvas,
                                        // then transforming the path itself by the inverse.
                                        let matrix = &gradient_transform.matrix;
                                        let _ = self.context.transform(
                                            matrix[0], matrix[1], matrix[2], matrix[3], matrix[4],
                                            matrix[5],
                                        );
                                        transform_dirty = true;
                                        let untransformed_path =
                                            Path2d::new().expect("Path2d constructor must succeed");
                                        untransformed_path.add_path_with_transformation(
                                            path,
                                            gradient_transform.inverse_matrix.unchecked_ref(),
                                        );
                                        self.context.fill_with_path_2d_and_winding(
                                            &untransformed_path,
                                            CanvasWindingRule::Evenodd,
                                        );
                                    } else {
                                        self.context.fill_with_path_2d_and_winding(
                                            path,
                                            CanvasWindingRule::Evenodd,
                                        );
                                    }

                                    self.clear_color_filter();
                                }
                                CanvasFillStyle::Bitmap(bitmap) => {
                                    self.set_color_filter(&transform);
                                    self.context.set_image_smoothing_enabled(bitmap.smoothed);
                                    self.context.set_fill_style(&bitmap.pattern);
                                    self.context.fill_with_path_2d_and_winding(
                                        path,
                                        CanvasWindingRule::Evenodd,
                                    );
                                    self.clear_color_filter();
                                }
                            }
                        }
                        CanvasDrawCommand::Stroke {
                            path,
                            line_width,
                            stroke_style,
                            line_cap,
                            line_join,
                            miter_limit,
                            scale_mode,
                        } => {
                            // Canvas.setTransform ends up transforming the stroke geometry itself (including joins/endcaps).
                            // Instead, reset the canvas transform, and apply the transform to the stroke path directly so
                            // that the geometry remains untransformed.
                            let _ = self.context.reset_transform();
                            transform_dirty = true;
                            let transformed_path =
                                Path2d::new().expect("Path2d constructor must succeed");
                            transformed_path
                                .add_path_with_transformation(path, dom_matrix.unchecked_ref());

                            // Set stroke parameters.
                            self.context.set_line_cap(line_cap);
                            self.context.set_line_join(line_join);
                            self.context.set_miter_limit(*miter_limit);
                            let line_width =
                                line_scale.transform_width(*line_width as f32, *scale_mode);
                            self.context.set_line_width(line_width.into());
                            match stroke_style {
                                CanvasStrokeStyle::Color(color) => {
                                    let color = color.color_transform(&transform.color_transform);
                                    self.context.set_stroke_style(&color.1.into());
                                    self.context.stroke_with_path(&transformed_path);
                                }
                                CanvasStrokeStyle::Gradient(gradient, focal_point) => {
                                    // This is the hard case -- the Canvas API provides no good way to transform gradients,
                                    // and the inverse-transform trick used above for gradient fills can't be used here
                                    // because it will distort the stroke geometry.
                                    // Another possibility is to avoid the Path2D API, instead drawing using explicit path
                                    // commands (`context.lineTo`), then push the gradient transform, and finally stroke
                                    // the path using `stroke()`. But this will be tons of JS calls if there are many strokes.
                                    // So let's settle for allocating a new canvas gradient that is a best-effort match of the
                                    // the desired transform. This will not match Flash exactly, but should be relatively rare.
                                    let mut gradient = gradient.clone();
                                    gradient.matrix =
                                        (transform.matrix * Matrix::from(gradient.matrix)).into();
                                    let gradient = match focal_point {
                                        Some(focal_point) => create_radial_gradient(
                                            &self.context,
                                            &gradient,
                                            *focal_point,
                                            false,
                                        ),
                                        None => {
                                            create_linear_gradient(&self.context, &gradient, false)
                                        }
                                    };
                                    if let Ok(gradient) = gradient {
                                        self.set_color_filter(&transform);
                                        self.context.set_stroke_style(&gradient.gradient);
                                        self.context.stroke_with_path(&transformed_path);
                                        self.clear_color_filter();
                                    }
                                }
                                CanvasStrokeStyle::Bitmap(bitmap) => {
                                    // Set the CanvasPattern's matrix to the concatenated transform.
                                    let bitmap_matrix = transform.matrix
                                        * bitmap.matrix
                                        * Matrix::scale(0.05, 0.05);
                                    bitmap.pattern.set_transform(
                                        bitmap_matrix.to_dom_matrix().unchecked_ref(),
                                    );
                                    self.set_color_filter(&transform);
                                    self.context.set_image_smoothing_enabled(bitmap.smoothed);
                                    self.context.set_stroke_style(&bitmap.pattern);
                                    self.context.stroke_with_path(&transformed_path);
                                    self.clear_color_filter();
                                }
                            };
                        }
                    }
                }
            }

            // Add the shape path to the mask path.
            // Strokes are ignored.
            MaskState::DrawMask(mask_path) => {
                for command in shape.0.iter() {
                    if let CanvasDrawCommand::Fill { path, .. } = command {
                        mask_path.add_path_with_transformation(
                            path,
                            transform.matrix.to_dom_matrix().unchecked_ref(),
                        );
                    }
                }
            }

            // Canvas backend doesn't have to do anything to clear masks.
            MaskState::ClearMask => (),
        }
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        match &self.mask_state {
            MaskState::DrawContent => {
                self.set_transform(&matrix);
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

    fn draw_line(&mut self, color: Color, matrix: Matrix) {
        self.draw_lines(color, matrix, false);
    }

    fn draw_line_rect(&mut self, color: Color, matrix: Matrix) {
        self.draw_lines(color, matrix, true)
    }

    fn push_mask(&mut self) {
        if self.mask_state == MaskState::DrawContent {
            // Save the current mask layer so that it can be restored when the mask is popped.
            self.context.save();
            self.mask_state =
                MaskState::DrawMask(Path2d::new().expect("Path2d constructor must succeed"));
        }
    }

    fn activate_mask(&mut self) {
        if let MaskState::DrawMask(mask_path) = &self.mask_state {
            self.context.reset_transform().warn_on_error();
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
                .clip_with_path_2d_and_winding(mask_path, CanvasWindingRule::Nonzero);
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

    fn blend(&mut self, commands: CommandList, blend: RenderBlendMode) {
        self.push_blend_mode(blend);
        commands.execute(self);
        self.pop_blend_mode();
    }
}

/// Convert a series of `DrawCommands` to a `Path2d` shape.
///
/// The path can be optionally closed by setting `is_closed` to `true`.
///
/// The resulting path is in the shape's own coordinate space and needs to be
/// transformed to fit within the shape's bounds.
fn draw_commands_to_path2d(commands: &[DrawCommand], is_closed: bool) -> Path2d {
    let path = Path2d::new().expect("Path2d constructor must succeed");
    for command in commands {
        match command {
            DrawCommand::MoveTo(move_to) => {
                path.move_to(move_to.x.get().into(), move_to.y.get().into())
            }
            DrawCommand::LineTo(line_to) => {
                path.line_to(line_to.x.get().into(), line_to.y.get().into())
            }
            DrawCommand::QuadraticCurveTo { control, anchor } => path.quadratic_curve_to(
                control.x.get().into(),
                control.y.get().into(),
                anchor.x.get().into(),
                anchor.y.get().into(),
            ),
            DrawCommand::CubicCurveTo {
                control_a,
                control_b,
                anchor,
            } => path.bezier_curve_to(
                control_a.x.get().into(),
                control_a.y.get().into(),
                control_b.x.get().into(),
                control_b.y.get().into(),
                anchor.x.get().into(),
                anchor.y.get().into(),
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
    backend: &mut WebCanvasRenderBackend,
) -> Vec<CanvasDrawCommand> {
    use ruffle_render::shape_utils::DrawPath;
    use swf::{FillStyle, LineCapStyle, LineJoinStyle};

    // Some browsers will vomit if you try to load/draw an image with 0 width/height.
    // TODO(Herschel): Might be better to just return None in this case and skip
    // rendering altogether.

    let mut canvas_data = vec![];

    let bounds_viewbox_matrix = DomMatrix::new().expect("DomMatrix constructor must succeed");
    bounds_viewbox_matrix.set_a(1.0 / 20.0);
    bounds_viewbox_matrix.set_d(1.0 / 20.0);

    for path in &shape.paths {
        match path {
            DrawPath::Fill {
                commands, style, ..
            } => {
                let canvas_path = Path2d::new().expect("Path2d constructor must succeed");
                canvas_path.add_path_with_transformation(
                    &draw_commands_to_path2d(commands, false),
                    bounds_viewbox_matrix.unchecked_ref(),
                );

                let fill_style = match style {
                    FillStyle::Color(color) => CanvasFillStyle::Color((*color).into()),
                    FillStyle::LinearGradient(gradient) => CanvasFillStyle::Gradient(
                        create_linear_gradient(&backend.context, gradient, true)
                            .expect("Couldn't create linear gradient"),
                    ),
                    FillStyle::RadialGradient(gradient) => CanvasFillStyle::Gradient(
                        create_radial_gradient(&backend.context, gradient, 0.0, true)
                            .expect("Couldn't create radial gradient"),
                    ),
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    } => CanvasFillStyle::Gradient(
                        create_radial_gradient(
                            &backend.context,
                            gradient,
                            focal_point.to_f64(),
                            true,
                        )
                        .expect("Couldn't create radial gradient"),
                    ),
                    FillStyle::Bitmap {
                        id,
                        matrix,
                        is_smoothed,
                        is_repeating,
                    } => {
                        let bitmap = if let Some(bitmap) = create_bitmap_pattern(
                            *id,
                            *matrix,
                            *is_smoothed,
                            *is_repeating,
                            bitmap_source,
                            backend,
                        ) {
                            bitmap
                        } else {
                            continue;
                        };
                        CanvasFillStyle::Bitmap(bitmap)
                    }
                };

                canvas_data.push(CanvasDrawCommand::Fill {
                    path: canvas_path,
                    fill_style,
                });
            }
            DrawPath::Stroke {
                commands,
                style,
                is_closed,
            } => {
                let canvas_path = Path2d::new().expect("Path2d constructor must succeed");
                canvas_path.add_path_with_transformation(
                    &draw_commands_to_path2d(commands, *is_closed),
                    bounds_viewbox_matrix.unchecked_ref(),
                );

                let stroke_style = match style.fill_style() {
                    FillStyle::Color(color) => CanvasStrokeStyle::Color((*color).into()),
                    FillStyle::LinearGradient(gradient) => {
                        CanvasStrokeStyle::Gradient(gradient.clone(), None)
                    }
                    FillStyle::RadialGradient(gradient) => {
                        CanvasStrokeStyle::Gradient(gradient.clone(), Some(0.0))
                    }
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    } => CanvasStrokeStyle::Gradient(gradient.clone(), Some(focal_point.to_f64())),
                    FillStyle::Bitmap {
                        id,
                        matrix,
                        is_smoothed,
                        is_repeating,
                    } => {
                        let bitmap = if let Some(bitmap) = create_bitmap_pattern(
                            *id,
                            *matrix,
                            *is_smoothed,
                            *is_repeating,
                            bitmap_source,
                            backend,
                        ) {
                            bitmap
                        } else {
                            continue;
                        };
                        CanvasStrokeStyle::Bitmap(bitmap)
                    }
                };

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
                canvas_data.push(CanvasDrawCommand::Stroke {
                    path: canvas_path,
                    line_width: style.width().to_pixels(),
                    stroke_style,
                    line_cap: line_cap.to_string(),
                    line_join: line_join.to_string(),
                    miter_limit: miter_limit as f64 / 20.0,
                    scale_mode: match (style.allow_scale_x(), style.allow_scale_y()) {
                        (false, false) => LineScaleMode::None,
                        (true, false) => LineScaleMode::Horizontal,
                        (false, true) => LineScaleMode::Vertical,
                        (true, true) => LineScaleMode::Both,
                    },
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
) -> Result<Gradient, JsError> {
    // Canvas linear gradients are configured via the line endpoints, so we only need
    // to transform it if the basis is not orthogonal (skew in the transform).
    let transformed = if is_fill {
        let dot = gradient.matrix.a * gradient.matrix.c + gradient.matrix.b * gradient.matrix.d;
        dot.to_f32().abs() > GRADIENT_TRANSFORM_THRESHOLD
    } else {
        // Complex gradient transforms can't apply to strokes; fall back to simple transforms.
        false
    };
    let create_fn = |matrix: swf::Matrix, gradient_scale: f64| {
        let start = matrix * Point::new(Twips::new(-16384), Twips::ZERO);
        let end = matrix * Point::new(Twips::new(16384), Twips::ZERO);
        // If we have to scale the gradient due to spread mode, scale the endpoints away from the center.
        let delta = end - start;
        let dx = 0.5 * (gradient_scale - 1.0) * delta.dx.to_pixels();
        let dy = 0.5 * (gradient_scale - 1.0) * delta.dy.to_pixels();
        Ok(context.create_linear_gradient(
            start.x.to_pixels() - dx,
            start.y.to_pixels() - dy,
            end.x.to_pixels() + dx,
            end.y.to_pixels() + dy,
        ))
    };
    swf_to_canvas_gradient(gradient, transformed, create_fn)
}

fn create_radial_gradient(
    context: &CanvasRenderingContext2d,
    gradient: &swf::Gradient,
    focal_point: f64,
    is_fill: bool,
) -> Result<Gradient, JsError> {
    // Canvas radial gradients can not be elliptical or skewed, so transform if there
    // is a non-uniform scale or skew.
    // A scale rotation matrix is always of the form:
    // [[a  b]
    //  [-b a]]
    let transformed = if is_fill {
        (gradient.matrix.a - gradient.matrix.d).to_f32().abs() > GRADIENT_TRANSFORM_THRESHOLD
            || (gradient.matrix.b + gradient.matrix.c).to_f32().abs() > GRADIENT_TRANSFORM_THRESHOLD
    } else {
        // Complex gradient transforms can't apply to strokes; fall back to simple transforms.
        false
    };
    let create_fn = |matrix: swf::Matrix, gradient_scale: f64| {
        let focal_center = matrix
            * Point::new(
                Twips::new((focal_point.clamp(-0.98, 0.98) * 16384.0) as i32),
                Twips::ZERO,
            );
        let center = matrix * Point::ZERO;
        let end = matrix * Point::new(Twips::new(16384), Twips::ZERO);
        let delta = end - center;
        let dx = delta.dx.to_pixels();
        let dy = delta.dy.to_pixels();
        let radius = (dx * dx + dy * dy).sqrt();
        context
            .create_radial_gradient(
                focal_center.x.to_pixels(),
                focal_center.y.to_pixels(),
                0.0,
                center.x.to_pixels(),
                center.y.to_pixels(),
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
) -> Result<Gradient, JsError> {
    let matrix = if transformed {
        // When we are rendering a complex gradient, the gradient transform is handled later by
        // transforming the path before rendering; so use the identity matrix here.
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
            if let (Some(first_stop), Some(last_stop)) = (color_stops.first(), color_stops.last()) {
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
    }

    Ok(Gradient {
        gradient: canvas_gradient,
        transform: transformed.then(|| {
            // When we render this gradient, we will push the gradient's transform to the canvas,
            // and then transform the path itself by the inverse.
            let matrix = swf_gradient.matrix.to_dom_matrix();
            let inverse_matrix = matrix.inverse();
            GradientTransform {
                matrix: [
                    matrix.a(),
                    matrix.b(),
                    matrix.c(),
                    matrix.d(),
                    matrix.e(),
                    matrix.f(),
                ],
                inverse_matrix,
            }
        }),
    })
}

/// Converts an SWF bitmap fill to a canvas pattern.
fn create_bitmap_pattern(
    id: swf::CharacterId,
    matrix: swf::Matrix,
    is_smoothed: bool,
    is_repeating: bool,
    bitmap_source: &dyn BitmapSource,
    backend: &mut WebCanvasRenderBackend,
) -> Option<CanvasBitmap> {
    if let Some(handle) = bitmap_source.bitmap_handle(id, backend) {
        let bitmap = as_bitmap_data(&handle);
        let repeat = if !is_repeating {
            // NOTE: The WebGL backend does clamping in this case, just like
            // Flash Player, but CanvasPattern has no such option...
            "no-repeat"
        } else {
            "repeat"
        };

        let pattern = match backend
            .context
            .create_pattern_with_html_canvas_element(&bitmap.canvas, repeat)
        {
            Ok(Some(pattern)) => pattern,
            _ => {
                log::warn!("Unable to create bitmap pattern for bitmap ID {}", id);
                return None;
            }
        };
        pattern.set_transform(matrix.to_dom_matrix().unchecked_ref());
        Some(CanvasBitmap {
            pattern,
            matrix: matrix.into(),
            smoothed: is_smoothed,
        })
    } else {
        log::warn!("Couldn't fill shape with unknown bitmap {}", id);
        None
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
        .expect("DomMatrix constructor must succeed")
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
        .expect("DomMatrix constructor must succeed")
    }
}

fn same_blend_mode(first: Option<&RenderBlendMode>, second: &RenderBlendMode) -> bool {
    match (first, second) {
        (Some(RenderBlendMode::Builtin(old)), RenderBlendMode::Builtin(new)) => old == new,
        _ => false,
    }
}
