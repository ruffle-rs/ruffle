use ruffle_core::backend::render::{
    swf::{self, CharacterId, GradientInterpolation, GradientSpread},
    Bitmap, BitmapFormat, BitmapHandle, BitmapInfo, Color, JpegTagFormat, Letterbox, RenderBackend,
    ShapeHandle, Transform,
};
use ruffle_core::color_transform::ColorTransform;
use ruffle_core::shape_utils::{DistilledShape, DrawCommand};
use ruffle_core::swf::Matrix;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;
use svg::node::element::{Group, Image, Rectangle};
use svg::{Document, Node};

type Error = Box<dyn std::error::Error>;

pub struct SvgRenderBackend {
    document: Document,
    shapes: Vec<Group>,
    bitmaps: Vec<BitmapData>,
    id_to_bitmap: HashMap<CharacterId, BitmapHandle>,
    viewport_width: u32,
    viewport_height: u32,
}

trait MatrixExt {
    fn is_identity(&self) -> bool;
    fn get_transform(&self) -> String;
    fn transform_element(&self, node: &mut impl Node) {
        if !self.is_identity() {
            node.assign("transform", format!("matrix({})", self.get_transform()));
        }
    }
}

impl MatrixExt for Matrix {
    #[allow(clippy::float_cmp)]
    fn is_identity(&self) -> bool {
        self.a == 1.0
            && self.b == 0.0
            && self.c == 0.0
            && self.d == 1.0
            && self.tx.get() == 0
            && self.ty.get() == 0
    }

    fn get_transform(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.a,
            self.b,
            self.c,
            self.d,
            self.tx.to_pixels(),
            self.ty.to_pixels(),
        )
    }
}

trait ColorTransformExt {
    fn is_identity(&self) -> bool;
    fn to_filter(&self) -> String;
}

impl ColorTransformExt for ColorTransform {
    #[allow(clippy::float_cmp)]
    fn is_identity(&self) -> bool {
        self.r_mult == 1.0
            && self.g_mult == 1.0
            && self.b_mult == 1.0
            && self.a_mult == 1.0
            && self.r_add == 0.0
            && self.g_add == 0.0
            && self.b_add == 0.0
            && self.a_add == 0.0
    }

    fn to_filter(&self) -> String {
        format!(
            "{} 0 0 0 {} 0 {} 0 0 {} 0 0 {} 0 {} 0 0 0 {} {}",
            self.r_mult,
            self.r_add,
            self.g_mult,
            self.g_add,
            self.b_mult,
            self.b_add,
            self.a_mult,
            self.a_add
        )
    }
}

#[allow(dead_code)]
struct BitmapData {
    width: u32,
    height: u32,
    data: String,
}

impl SvgRenderBackend {
    fn new_document(width: u32, height: u32) -> Document {
        Document::new()
            .set("preserveAspectRatio", "none")
            .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("width", width)
            .set("height", height)
            .set("viewBox", (0, 0, width, height))
    }

    pub fn new(width: u32, height: u32) -> Self {
        Self {
            document: SvgRenderBackend::new_document(width, height),
            shapes: vec![],
            bitmaps: vec![],
            id_to_bitmap: HashMap::new(),
            viewport_width: width,
            viewport_height: height,
        }
    }

    /// Converts an RGBA image into a PNG encoded as a base64 data URI.
    fn bitmap_to_png_data_uri(bitmap: Bitmap) -> Result<String, Box<dyn std::error::Error>> {
        use png::Encoder;
        let mut png_data: Vec<u8> = vec![];
        {
            let mut encoder = Encoder::new(&mut png_data, bitmap.width, bitmap.height);
            encoder.set_depth(png::BitDepth::Eight);
            let data = match bitmap.data {
                BitmapFormat::Rgba(mut data) => {
                    ruffle_core::backend::render::unmultiply_alpha_rgba(&mut data[..]);
                    encoder.set_color(png::ColorType::RGBA);
                    data
                }
                BitmapFormat::Rgb(data) => {
                    encoder.set_color(png::ColorType::RGB);
                    data
                }
            };
            let mut writer = encoder.write_header()?;
            writer.write_image_data(&data)?;
        }

        Ok(format!(
            "data:image/png;base64,{}",
            &base64::encode(&png_data[..])
        ))
    }

    fn register_bitmap_pure_jpeg(
        &mut self,
        id: CharacterId,
        data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        let data = ruffle_core::backend::render::remove_invalid_jpeg_data(data);
        let mut decoder = jpeg_decoder::Decoder::new(&data[..]);
        decoder.read_info().unwrap();
        let metadata = decoder.info().unwrap();

        let jpeg_encoded = format!("data:image/jpeg;base64,{}", &base64::encode(&data[..]));

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            width: metadata.width.into(),
            height: metadata.height.into(),
            data: jpeg_encoded,
        });
        self.id_to_bitmap.insert(id, handle);
        Ok(BitmapInfo {
            handle,
            width: metadata.width,
            height: metadata.height,
        })
    }

    fn register_bitmap_raw(
        &mut self,
        id: CharacterId,
        bitmap: Bitmap,
    ) -> Result<BitmapInfo, Error> {
        let (width, height) = (bitmap.width, bitmap.height);
        let png = Self::bitmap_to_png_data_uri(bitmap)?;

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            width,
            height,
            data: png,
        });

        self.id_to_bitmap.insert(id, handle);
        Ok(BitmapInfo {
            handle,
            width: width.try_into().expect("JPEG dimensions too large"),
            height: height.try_into().expect("JPEG dimensions too large"),
        })
    }
}

impl fmt::Display for SvgRenderBackend {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.document)
    }
}

impl RenderBackend for SvgRenderBackend {
    fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
        dbg!(width, height);
        self.document.assign("width", width);
        self.document.assign("height", height);
        self.document.assign("viewBox", (0, 0, width, height));
    }

    fn register_shape(&mut self, shape: DistilledShape) -> ShapeHandle {
        let handle = ShapeHandle(self.shapes.len());

        let mut bitmaps = HashMap::new();
        for (id, handle) in &self.id_to_bitmap {
            let bitmap_data = &self.bitmaps[handle.0];
            bitmaps.insert(
                *id,
                (&bitmap_data.data[..], bitmap_data.width, bitmap_data.height),
            );
        }

        let data = swf_shape_to_svg(shape, &bitmaps, "crisp-edges");
        self.shapes.push(data);

        handle
    }

    fn replace_shape(&mut self, shape: DistilledShape, handle: ShapeHandle) {
        let mut bitmaps = HashMap::new();
        for (id, handle) in &self.id_to_bitmap {
            let bitmap_data = &self.bitmaps[handle.0];
            bitmaps.insert(
                *id,
                (&bitmap_data.data[..], bitmap_data.width, bitmap_data.height),
            );
        }

        let data = swf_shape_to_svg(shape, &bitmaps, "crisp-edges");
        self.shapes[handle.0] = data;
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let shape = ruffle_core::shape_utils::swf_glyph_to_shape(glyph);
        self.register_shape((&shape).into())
    }

    fn register_bitmap_jpeg(
        &mut self,
        id: CharacterId,
        data: &[u8],
        jpeg_tables: Option<&[u8]>,
    ) -> Result<BitmapInfo, Error> {
        let data = ruffle_core::backend::render::glue_tables_to_jpeg(data, jpeg_tables);
        self.register_bitmap_pure_jpeg(id, &data)
    }

    fn register_bitmap_jpeg_2(
        &mut self,
        id: CharacterId,
        data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        if ruffle_core::backend::render::determine_jpeg_tag_format(data) == JpegTagFormat::Jpeg {
            self.register_bitmap_pure_jpeg(id, data)
        } else {
            let bitmap = ruffle_core::backend::render::decode_define_bits_jpeg(data, None)?;
            self.register_bitmap_raw(id, bitmap)
        }
    }

    fn register_bitmap_jpeg_3(
        &mut self,
        id: swf::CharacterId,
        jpeg_data: &[u8],
        alpha_data: &[u8],
    ) -> Result<BitmapInfo, Error> {
        let bitmap =
            ruffle_core::backend::render::decode_define_bits_jpeg(jpeg_data, Some(alpha_data))?;
        self.register_bitmap_raw(id, bitmap)
    }

    fn register_bitmap_png(
        &mut self,
        swf_tag: &swf::DefineBitsLossless,
    ) -> Result<BitmapInfo, Error> {
        let bitmap = ruffle_core::backend::render::decode_define_bits_lossless(swf_tag)?;

        let png = Self::bitmap_to_png_data_uri(bitmap)?;

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            width: swf_tag.width.into(),
            height: swf_tag.height.into(),
            data: png,
        });
        self.id_to_bitmap.insert(swf_tag.id, handle);
        Ok(BitmapInfo {
            handle,
            width: swf_tag.width,
            height: swf_tag.height,
        })
    }

    fn begin_frame(&mut self, clear: Color) {
        self.document = SvgRenderBackend::new_document(self.viewport_width, self.viewport_height);
        let color = format!("rgb({}, {}, {})", clear.r, clear.g, clear.b);
        self.document.append(
            Rectangle::new()
                .set("width", self.viewport_width)
                .set("height", self.viewport_height)
                .set("color", color.as_str()),
        );
    }

    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: &Transform) {
        if let Some(bitmap) = self.bitmaps.get(bitmap.0) {
            assert!(transform.color_transform.is_identity());
            let mut image = Image::new()
                .set("width", bitmap.width)
                .set("height", bitmap.height)
                .set("href", bitmap.data.as_str());
            transform.matrix.transform_element(&mut image);
            self.document.append(image);
        }
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        if let Some(shape) = self.shapes.get(shape.0) {
            assert!(transform.color_transform.is_identity());
            let mut shape = shape.clone();
            if !transform.matrix.is_identity() {
                shape = Group::new()
                    .set(
                        "transform",
                        format!("matrix({})", transform.matrix.get_transform()),
                    )
                    .add(shape);
            }
            self.document.append(shape);
        }
    }

    fn draw_rect(&mut self, Color { r, g, b, a }: Color, matrix: &Matrix) {
        self.document.append(
            Rectangle::new()
                .set("width", 1)
                .set("height", 1)
                .set("transform", format!("matrix({})", matrix.get_transform()))
                .set("fill", format!("rgb({},{},{})", r, g, b))
                .set("fill-opacity", f32::from(a) / 255.0),
        );
    }

    fn end_frame(&mut self) {}

    fn draw_letterbox(&mut self, letterbox: Letterbox) {
        match letterbox {
            Letterbox::None => (),
            Letterbox::Letterbox(margin_height) => {
                self.document.append(
                    Rectangle::new()
                        .set("width", self.viewport_width)
                        .set("height", margin_height),
                );
                self.document.append(
                    Rectangle::new()
                        .set("y", self.viewport_height as f32 - margin_height)
                        .set("width", self.viewport_width)
                        .set("height", self.viewport_height),
                );
            }
            Letterbox::Pillarbox(margin_width) => {
                self.document.append(
                    Rectangle::new()
                        .set("width", margin_width)
                        .set("height", self.viewport_height),
                );
                self.document.append(
                    Rectangle::new()
                        .set("x", self.viewport_width as f32 - margin_width)
                        .set("width", margin_width)
                        .set("height", self.viewport_height),
                );
            }
        }
    }

    fn push_mask(&mut self) {
        // todo!()
    }
    fn activate_mask(&mut self) {
        // todo!()
    }
    fn deactivate_mask(&mut self) {
        // todo!()
    }
    fn pop_mask(&mut self) {
        // todo!()
    }
}

#[allow(clippy::cognitive_complexity)]
fn swf_shape_to_svg(
    shape: DistilledShape,
    bitmaps: &HashMap<CharacterId, (&str, u32, u32)>,
    pixelated_property_value: &str,
) -> Group {
    use fnv::FnvHashSet;
    use ruffle_core::shape_utils::DrawPath;
    use svg::node::element::{
        path::Data, Definitions, Filter, LinearGradient, Path as SvgPath, Pattern, RadialGradient,
        Stop,
    };
    use swf::{FillStyle, LineCapStyle, LineJoinStyle};

    let (svg_width, svg_height) = (
        (shape.shape_bounds.x_max - shape.shape_bounds.x_min).to_pixels() as f64,
        (shape.shape_bounds.y_max - shape.shape_bounds.y_min).to_pixels() as f64,
    );
    let mut document = Document::new()
        .set("width", svg_width)
        .set("height", svg_height)
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
    let mut has_linear_rgb_gradient = false;

    let mut svg_paths = vec![];
    for path in shape.paths {
        match path {
            DrawPath::Fill { style, commands } => {
                let mut svg_path = SvgPath::new();

                let (fill, opacity) = match style {
                    FillStyle::Color(Color { r, g, b, a }) => {
                        (format!("rgb({},{},{})", r, g, b), f32::from(*a) / 255.0)
                    }
                    FillStyle::LinearGradient(gradient) => {
                        let shift = Matrix {
                            a: 32768.0 / width,
                            d: 32768.0 / height,
                            tx: swf::Twips::new(-16384),
                            ty: swf::Twips::new(-16384),
                            ..Default::default()
                        };
                        let gradient_matrix = gradient.matrix * shift;

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
                        svg_gradient = match gradient.spread {
                            GradientSpread::Pad => svg_gradient, // default
                            GradientSpread::Reflect => svg_gradient.set("spreadMethod", "reflect"),
                            GradientSpread::Repeat => svg_gradient.set("spreadMethod", "repeat"),
                        };
                        if gradient.interpolation == GradientInterpolation::LinearRGB {
                            has_linear_rgb_gradient = true;
                            svg_path = svg_path.set("filter", "url('#_linearrgb')");
                        }
                        for record in &gradient.records {
                            let color =
                                if gradient.interpolation == GradientInterpolation::LinearRGB {
                                    srgb_to_linear(record.color.clone())
                                } else {
                                    record.color.clone()
                                };
                            let stop = Stop::new()
                                .set("offset", format!("{}%", f32::from(record.ratio) / 2.55))
                                .set(
                                    "stop-color",
                                    format!(
                                        "rgba({},{},{},{})",
                                        color.r,
                                        color.g,
                                        color.b,
                                        f32::from(color.a) / 255.0
                                    ),
                                );
                            svg_gradient = svg_gradient.add(stop);
                        }
                        defs = defs.add(svg_gradient);

                        let fill_id = format!("url(#f{})", num_defs);
                        num_defs += 1;
                        (fill_id, 1.0)
                    }
                    FillStyle::RadialGradient(gradient) => {
                        let shift = Matrix {
                            a: 32768.0,
                            d: 32768.0,
                            ..Default::default()
                        };
                        let gradient_matrix = gradient.matrix * shift;

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
                        svg_gradient = match gradient.spread {
                            GradientSpread::Pad => svg_gradient, // default
                            GradientSpread::Reflect => svg_gradient.set("spreadMethod", "reflect"),
                            GradientSpread::Repeat => svg_gradient.set("spreadMethod", "repeat"),
                        };
                        if gradient.interpolation == GradientInterpolation::LinearRGB {
                            has_linear_rgb_gradient = true;
                            svg_path = svg_path.set("filter", "url('#_linearrgb')");
                        }
                        for record in &gradient.records {
                            let color =
                                if gradient.interpolation == GradientInterpolation::LinearRGB {
                                    srgb_to_linear(record.color.clone())
                                } else {
                                    record.color.clone()
                                };
                            let stop = Stop::new()
                                .set("offset", format!("{}%", f32::from(record.ratio) / 2.55))
                                .set(
                                    "stop-color",
                                    format!(
                                        "rgba({},{},{},{})",
                                        color.r,
                                        color.g,
                                        color.b,
                                        f32::from(color.a) / 255.0
                                    ),
                                );
                            svg_gradient = svg_gradient.add(stop);
                        }
                        defs = defs.add(svg_gradient);

                        let fill_id = format!("url(#f{})", num_defs);
                        num_defs += 1;
                        (fill_id, 1.0)
                    }
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    } => {
                        let shift = Matrix {
                            a: 32768.0,
                            d: 32768.0,
                            ..Default::default()
                        };
                        let gradient_matrix = gradient.matrix * shift;

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
                        svg_gradient = match gradient.spread {
                            GradientSpread::Pad => svg_gradient, // default
                            GradientSpread::Reflect => svg_gradient.set("spreadMethod", "reflect"),
                            GradientSpread::Repeat => svg_gradient.set("spreadMethod", "repeat"),
                        };
                        if gradient.interpolation == GradientInterpolation::LinearRGB {
                            has_linear_rgb_gradient = true;
                            svg_path = svg_path.set("filter", "url('#_linearrgb')");
                        }
                        for record in &gradient.records {
                            let color =
                                if gradient.interpolation == GradientInterpolation::LinearRGB {
                                    srgb_to_linear(record.color.clone())
                                } else {
                                    record.color.clone()
                                };
                            let stop = Stop::new()
                                .set("offset", format!("{}%", f32::from(record.ratio) / 2.55))
                                .set(
                                    "stop-color",
                                    format!(
                                        "rgba({},{},{},{})",
                                        color.r,
                                        color.g,
                                        color.b,
                                        f32::from(color.a) / 255.0
                                    ),
                                );
                            svg_gradient = svg_gradient.add(stop);
                        }
                        defs = defs.add(svg_gradient);

                        let fill_id = format!("url(#f{})", num_defs);
                        num_defs += 1;
                        (fill_id, 1.0)
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

                        let svg_pattern = Pattern::new()
                            .set("id", format!("f{}", num_defs))
                            .set("xlink:href", format!("#b{}", id))
                            .set(
                                "patternTransform",
                                format!(
                                    "matrix({} {} {} {} {} {})",
                                    matrix.a,
                                    matrix.b,
                                    matrix.c,
                                    matrix.d,
                                    matrix.tx.get(),
                                    matrix.ty.get()
                                ),
                            );

                        defs = defs.add(svg_pattern);

                        let fill_id = format!("url(#f{})", num_defs);
                        num_defs += 1;
                        (fill_id, 1.0)
                    }
                };
                svg_path = svg_path.set("fill", fill).set("fill-opacity", opacity);

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

    // If this shape contains a gradient in linear RGB space, add a filter to do the color space adjustment.
    // We have to use a filter because browser don't seem to implement the `color-interpolation` SVG property.
    if has_linear_rgb_gradient {
        // Add a filter to convert from linear space to sRGB space.
        let mut filter = Filter::new();
        filter = filter.set("id", "_linearrgb");
        filter = filter.set("color-interpolation-filters", "sRGB");
        let text = svg::node::Text::new(
            r#"
            <feComponentTransfer>
                <feFuncR type="gamma" exponent="0.4545454545"></feFuncR>
                <feFuncG type="gamma" exponent="0.4545454545"></feFuncG>
                <feFuncB type="gamma" exponent="0.4545454545"></feFuncB>
            </feComponentTransfer>
            "#,
        );
        filter = filter.add(text);
        defs = defs.add(filter);
        num_defs += 1;
    }

    if num_defs > 0 {
        document = document.add(defs);
    }

    for svg_path in svg_paths {
        document = document.add(svg_path);
    }

    Group::new().add(document).set(
        "transform",
        format!(
            "translate({} {})",
            shape.shape_bounds.x_min.to_pixels(),
            shape.shape_bounds.y_min.to_pixels()
        )
        .as_str(),
    )
}

/// Converts an SWF color from sRGB space to linear color space.
pub fn srgb_to_linear(color: swf::Color) -> swf::Color {
    ruffle_core::backend::render::srgb_to_linear(color.into()).into()
}
