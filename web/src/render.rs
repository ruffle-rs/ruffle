use ruffle_core::backend::render::{
    swf, swf::CharacterId, BitmapHandle, Color, RenderBackend, ShapeHandle, Transform,
};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, HtmlImageElement};

pub struct WebCanvasRenderBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    color_matrix: Element,
    shapes: Vec<ShapeData>,
    bitmaps: Vec<BitmapData>,
    id_to_bitmap: HashMap<CharacterId, BitmapHandle>,
}

struct ShapeData {
    image: HtmlImageElement,
    x_min: f64,
    y_min: f64,
}

#[allow(dead_code)]
struct BitmapData {
    image: HtmlImageElement,
    width: u32,
    height: u32,
    data: String,
}

impl WebCanvasRenderBackend {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, Box<std::error::Error>> {
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .map_err(|_| "Could not create context")?
            .ok_or("Could not create context")?
            .dyn_into()
            .map_err(|_| "Expected CanvasRenderingContext2d")?;

        let document = web_sys::window().unwrap().document().unwrap();
        let svg = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
            .map_err(|_| "Couldn't make SVG")?;

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

        // Ensure a previous instance of the color matrix filter node doesn't exist.
        // TODO: Remove it in player.destroy()? This is dangerous if the client page has something with this id...
        if let Some(element) = document.get_element_by_id("_cm") {
            element.remove();
        }

        // Create a color matrix filter to handle Flash color effects.
        let filter = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "filter")
            .map_err(|_| "Couldn't make SVG filter")?;
        filter
            .set_attribute("id", "_cm")
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

        canvas
            .append_child(&filter)
            .map_err(|_| "append_child failed")?;

        Ok(Self {
            canvas: canvas.clone(),
            color_matrix,
            context,
            shapes: vec![],
            bitmaps: vec![],
            id_to_bitmap: HashMap::new(),
        })
    }
}

impl RenderBackend for WebCanvasRenderBackend {
    fn set_movie_dimensions(&mut self, _width: u32, _height: u32) {}
    fn set_viewport_dimensions(&mut self, _width: u32, _height: u32) {}

    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.shapes.len());

        let image = HtmlImageElement::new().unwrap();

        let mut bitmaps = HashMap::new();
        for (id, handle) in &self.id_to_bitmap {
            let bitmap_data = &self.bitmaps[handle.0];
            bitmaps.insert(
                *id,
                (&bitmap_data.data[..], bitmap_data.width, bitmap_data.height),
            );
        }

        use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
        let svg = swf_shape_to_svg(&shape, &bitmaps);

        let svg_encoded = format!(
            "data:image/svg+xml,{}",
            utf8_percent_encode(&svg, DEFAULT_ENCODE_SET) //&base64::encode(&svg[..])
        );

        image.set_src(&svg_encoded);

        self.shapes.push(ShapeData {
            image,
            x_min: shape.shape_bounds.x_min.to_pixels(),
            y_min: shape.shape_bounds.y_min.to_pixels(),
        });

        handle
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let bounds = glyph.bounds.clone().unwrap_or_else(|| {
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
        mut data: &[u8],
        mut jpeg_tables: &[u8],
    ) -> BitmapHandle {
        // SWF19 p.138:
        // "Before version 8 of the SWF file format, SWF files could contain an erroneous header of 0xFF, 0xD9, 0xFF, 0xD8 before the JPEG SOI marker."
        // Slice off these bytes if necessary.`
        if data[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            data = &data[4..];
        }

        if jpeg_tables[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            jpeg_tables = &jpeg_tables[4..];
        }

        let mut full_jpeg = jpeg_tables[..jpeg_tables.len() - 2].to_vec();
        full_jpeg.extend_from_slice(&data[2..]);

        self.register_bitmap_jpeg_2(id, &full_jpeg[..])
    }

    fn register_bitmap_jpeg_2(&mut self, id: CharacterId, mut data: &[u8]) -> BitmapHandle {
        // SWF19 p.138:
        // "Before version 8 of the SWF file format, SWF files could contain an erroneous header of 0xFF, 0xD9, 0xFF, 0xD8 before the JPEG SOI marker."
        // Slice off these bytes if necessary.`
        if data[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            data = &data[4..];
        }

        let mut decoder = jpeg_decoder::Decoder::new(data);
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
        handle
    }

    fn register_bitmap_png(&mut self, swf_tag: &swf::DefineBitsLossless) -> BitmapHandle {
        use inflate::inflate_bytes_zlib;
        let mut decoded_data = inflate_bytes_zlib(&swf_tag.data).unwrap();
        match (swf_tag.version, swf_tag.format) {
            (1, swf::BitmapFormat::Rgb15) => unimplemented!("15-bit PNG"),
            (1, swf::BitmapFormat::Rgb32) => {
                let mut i = 0;
                while i < decoded_data.len() {
                    decoded_data[i] = decoded_data[i + 1];
                    decoded_data[i + 1] = decoded_data[i + 2];
                    decoded_data[i + 2] = decoded_data[i + 3];
                    decoded_data[i + 3] = 0xff;
                    i += 4;
                }
            }
            (2, swf::BitmapFormat::Rgb32) => {
                let mut i = 0;
                while i < decoded_data.len() {
                    let alpha = decoded_data[i];
                    decoded_data[i] = decoded_data[i + 1];
                    decoded_data[i + 1] = decoded_data[i + 2];
                    decoded_data[i + 2] = decoded_data[i + 3];
                    decoded_data[i + 3] = alpha;
                    i += 4;
                }
            }
            (2, swf::BitmapFormat::ColorMap8) => {
                let mut i = 0;
                let padded_width = (swf_tag.width + 0b11) & !0b11;

                let mut palette = Vec::with_capacity(swf_tag.num_colors as usize + 1);
                for _ in 0..=swf_tag.num_colors {
                    palette.push(Color {
                        r: decoded_data[i],
                        g: decoded_data[i + 1],
                        b: decoded_data[i + 2],
                        a: decoded_data[i + 3],
                    });
                    i += 4;
                }
                let mut out_data = vec![];
                for _ in 0..swf_tag.height {
                    for _ in 0..swf_tag.width {
                        let entry = decoded_data[i] as usize;
                        if entry < palette.len() {
                            let color = &palette[entry];
                            out_data.push(color.r);
                            out_data.push(color.g);
                            out_data.push(color.b);
                            out_data.push(color.a);
                        } else {
                            out_data.push(0);
                            out_data.push(0);
                            out_data.push(0);
                            out_data.push(0);
                        }
                        i += 1;
                    }
                    i += (padded_width - swf_tag.width) as usize;
                }
                decoded_data = out_data;
            }
            _ => unimplemented!(),
        }

        let mut out_png: Vec<u8> = vec![];
        {
            use png::{Encoder, HasParameters};
            let mut encoder =
                Encoder::new(&mut out_png, swf_tag.width.into(), swf_tag.height.into());
            encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&decoded_data).unwrap();
        }

        let image = HtmlImageElement::new().unwrap();
        let png_encoded = format!("data:image/png;base64,{}", &base64::encode(&out_png[..]));

        let handle = BitmapHandle(self.bitmaps.len());
        self.bitmaps.push(BitmapData {
            image,
            width: swf_tag.width.into(),
            height: swf_tag.height.into(),
            data: png_encoded,
        });
        self.id_to_bitmap.insert(swf_tag.id, handle);
        handle
    }

    fn begin_frame(&mut self) {
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

    #[allow(clippy::float_cmp)]
    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let shape = if let Some(shape) = self.shapes.get(shape.0) {
            shape
        } else {
            return;
        };

        self.context
            .set_transform(
                transform.matrix.a.into(),
                transform.matrix.b.into(),
                transform.matrix.c.into(),
                transform.matrix.d.into(),
                f64::from(transform.matrix.tx) / 20.0,
                f64::from(transform.matrix.ty) / 20.0,
            )
            .unwrap();

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
            let matrix_str = format!(
                "{} 0 0 0 {} 0 {} 0 0 {} 0 0 {} 0 {} 0 0 0 {} {}",
                color_transform.r_mult,
                color_transform.r_add,
                color_transform.g_mult,
                color_transform.g_add,
                color_transform.b_mult,
                color_transform.b_add,
                color_transform.a_mult,
                color_transform.a_add
            );
            self.color_matrix
                .set_attribute("values", &matrix_str)
                .unwrap();

            self.context.set_filter("url('#_cm')");
        }

        self.context
            .draw_image_with_html_image_element(&shape.image, shape.x_min, shape.y_min)
            .unwrap();

        self.context.set_filter("none");
        self.context.set_global_alpha(1.0);
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
}

fn swf_shape_to_svg(
    shape: &swf::Shape,
    bitmaps: &HashMap<CharacterId, (&str, u32, u32)>,
) -> String {
    use fnv::FnvHashSet;
    use ruffle_core::matrix::Matrix;
    use ruffle_core::shape_utils::{swf_shape_to_paths, DrawCommand, DrawPath};
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
                                tx: -16384.0,
                                ty: -16384.0,
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
                                        gradient_matrix.tx,
                                        gradient_matrix.ty
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
                                a: 32768.0 / width,
                                d: 32768.0 / height,
                                tx: -16384.0,
                                ty: -16384.0,
                                ..Default::default()
                            };
                            let gradient_matrix = matrix * shift;

                            let mut svg_gradient = RadialGradient::new()
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
                                        gradient_matrix.tx,
                                        gradient_matrix.ty
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
                                            record.color.a
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
                                a: 32768.0 / width,
                                d: 32768.0 / height,
                                tx: -16384.0,
                                ty: -16384.0,
                                ..Default::default()
                            };
                            let gradient_matrix = matrix * shift;

                            let mut svg_gradient = RadialGradient::new()
                                .set("id", format!("f{}", num_defs))
                                .set("fx", -focal_point)
                                .set("gradientUnits", "userSpaceOnUse")
                                .set(
                                    "gradientTransform",
                                    format!(
                                        "matrix({} {} {} {} {} {})",
                                        gradient_matrix.a,
                                        gradient_matrix.b,
                                        gradient_matrix.c,
                                        gradient_matrix.d,
                                        gradient_matrix.tx,
                                        gradient_matrix.ty
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
                                            record.color.a
                                        ),
                                    );
                                svg_gradient = svg_gradient.add(stop);
                            }
                            defs = defs.add(svg_gradient);

                            let fill_id = format!("url(#f{})", num_defs);
                            num_defs += 1;
                            fill_id
                        }
                        FillStyle::Bitmap { id, matrix, .. } => {
                            let (bitmap_data, bitmap_width, bitmap_height) =
                                bitmaps.get(&id).unwrap_or(&("", 0, 0));

                            if !bitmap_defs.contains(&id) {
                                let image = Image::new()
                                    .set("width", *bitmap_width)
                                    .set("height", *bitmap_height)
                                    .set("xlink:href", *bitmap_data);

                                let bitmap_pattern = Pattern::new()
                                    .set("id", format!("b{}", id))
                                    .set("width", *bitmap_width)
                                    .set("height", *bitmap_height)
                                    .set("patternUnits", "userSpaceOnUse")
                                    .add(image);

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
                                        bitmap_matrix.tx,
                                        bitmap_matrix.ty
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
                    .set("stroke-width", style.width.get())
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

    document.to_string()
}
