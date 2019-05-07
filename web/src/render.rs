use fluster_core::backend::render::{
    swf, swf::CharacterId, BitmapHandle, Color, RenderBackend, ShapeHandle, Transform,
};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, HtmlImageElement};

pub struct WebCanvasRenderBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    svg_defs: Element,
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

        svg.set_attribute("width", "0");
        svg.set_attribute("height", "0");
        svg.set_attribute_ns(
            Some("http://www.w3.org/2000/xmlns/"),
            "xmlns:xlink",
            "http://www.w3.org/1999/xlink",
        )
        .map_err(|_| "Couldn't make SVG")?;

        let svg_defs = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "defs")
            .map_err(|_| "Couldn't make SVG defs")?;

        let filter = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "filter")
            .map_err(|_| "Couldn't make SVG filter")?;
        filter.set_attribute("id", "cm");

        let color_matrix = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "feColorMatrix")
            .map_err(|_| "Couldn't make SVG feColorMatrix element")?;
        color_matrix.set_attribute("type", "matrix");
        color_matrix.set_attribute("values", "1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0 0 1 0");
        // canvas
        //     .set_attribute(
        //         "style",
        //         "color-interpolation-filters:linearRGB;color-interpolation:linearRGB",
        //     )
        //     .unwrap();
        // color_matrix
        //     .set_attribute(
        //         "style",
        //         "color-interpolation-filters:linearRGB;color-interpolation:linearRGB",
        //     )
        //     .unwrap();
        // filter
        //     .set_attribute(
        //         "style",
        //         "color-interpolation-filters:linearRGB;color-interpolation:linearRGB",
        //     )
        //     .unwrap();
        filter
            .append_child(&color_matrix.clone())
            .map_err(|_| "append_child failed")?;
        svg_defs
            .append_child(&filter)
            .map_err(|_| "append_child failed")?;
        svg.append_child(&svg_defs.clone())
            .map_err(|_| "append_child failed")?;

        let body = document
            .body()
            .unwrap()
            .append_child(&svg)
            .map_err(|_| "append_child failed")?;

        Ok(Self {
            canvas: canvas.clone(),
            color_matrix,
            svg_defs,
            context,
            shapes: vec![],
            bitmaps: vec![],
            id_to_bitmap: HashMap::new(),
        })
    }
}

impl RenderBackend for WebCanvasRenderBackend {
    fn set_dimensions(&mut self, _width: u32, _height: u32) {}

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
        let svg = crate::shape_utils::swf_shape_to_svg(&shape, &bitmaps);

        let svg_encoded = format!(
            "data:image/svg+xml,{}",
            utf8_percent_encode(&svg, DEFAULT_ENCODE_SET) //&base64::encode(&svg[..])
        );

        image.set_src(&svg_encoded);

        self.shapes.push(ShapeData {
            image,
            x_min: shape.shape_bounds.x_min.into(),
            y_min: shape.shape_bounds.y_min.into(),
        });

        handle
    }

    fn register_glyph_shape(&mut self, glyph: &swf::Glyph) -> ShapeHandle {
        let bounds = glyph.bounds.clone().unwrap_or_else(|| {
            let mut bounds = swf::Rectangle {
                x_min: 999.0,
                x_max: -999.0,
                y_min: 999.0,
                y_max: -999.0,
            };
            let mut x = 0.0;
            let mut y = 0.0;
            for record in &glyph.shape_records {
                match record {
                    swf::ShapeRecord::StyleChange(style_change) => {
                        if let Some((move_x, move_y)) = style_change.move_to {
                            x = move_x;
                            y = move_y;
                            bounds.x_min = f32::min(bounds.x_min, x);
                            bounds.x_max = f32::max(bounds.x_max, x);
                            bounds.y_min = f32::min(bounds.y_min, y);
                            bounds.y_max = f32::max(bounds.y_max, y);
                        }
                    }
                    swf::ShapeRecord::StraightEdge { delta_x, delta_y } => {
                        x += delta_x;
                        y += delta_y;
                        bounds.x_min = f32::min(bounds.x_min, x);
                        bounds.x_max = f32::max(bounds.x_max, x);
                        bounds.y_min = f32::min(bounds.y_min, y);
                        bounds.y_max = f32::max(bounds.y_max, y);
                    }
                    swf::ShapeRecord::CurvedEdge {
                        control_delta_x,
                        control_delta_y,
                        anchor_delta_x,
                        anchor_delta_y,
                    } => {
                        x += control_delta_x;
                        y += control_delta_y;
                        bounds.x_min = f32::min(bounds.x_min, x);
                        bounds.x_max = f32::max(bounds.x_max, x);
                        bounds.y_min = f32::min(bounds.y_min, y);
                        bounds.y_max = f32::max(bounds.y_max, y);
                        x += anchor_delta_x;
                        y += anchor_delta_y;
                        bounds.x_min = f32::min(bounds.x_min, x);
                        bounds.x_max = f32::max(bounds.x_max, x);
                        bounds.y_min = f32::min(bounds.y_min, y);
                        bounds.y_max = f32::max(bounds.y_max, y);
                    }
                }
            }
            if bounds.x_max < bounds.x_min || bounds.y_max < bounds.y_min {
                bounds = swf::Rectangle {
                    x_min: 0.0,
                    x_max: 0.0,
                    y_min: 0.0,
                    y_max: 0.0,
                };
            }
            bounds
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
        if &data[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            data = &data[4..];
        }

        if &jpeg_tables[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            jpeg_tables = &jpeg_tables[4..];
        }

        let mut full_jpeg = jpeg_tables[..jpeg_tables.len() - 2].to_vec();
        full_jpeg.extend_from_slice(&data[2..]);

        self.register_bitmap_jpeg_2(id, &full_jpeg[..])
    }

    fn register_bitmap_jpeg_2(&mut self, id: CharacterId, mut data: &[u8]) -> BitmapHandle {
        use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

        // SWF19 p.138:
        // "Before version 8 of the SWF file format, SWF files could contain an erroneous header of 0xFF, 0xD9, 0xFF, 0xD8 before the JPEG SOI marker."
        // Slice off these bytes if necessary.`
        if &data[0..4] == [0xFF, 0xD9, 0xFF, 0xD8] {
            data = &data[4..];
        }

        let mut decoder = jpeg_decoder::Decoder::new(data);
        decoder.read_info().unwrap();
        let metadata = decoder.info().unwrap();

        let image = HtmlImageElement::new().unwrap();
        let jpeg_encoded = format!("data:image/jpeg;base64,{}", &base64::encode(&data[..]));
        image.set_src(&jpeg_encoded);

        let document = web_sys::window().unwrap().document().unwrap();

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
        let image = HtmlImageElement::new().unwrap();

        use std::io::{Read, Write};

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
                for _ in 0..swf_tag.num_colors + 1 {
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

        let color = format!("rgb({}, {}, {}", color.r, color.g, color.b);
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
                transform.matrix.tx.into(),
                transform.matrix.ty.into(),
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

            self.context.set_filter("url('#cm')");
        }

        self.context
            .draw_image_with_html_image_element(&shape.image, shape.x_min, shape.y_min)
            .unwrap();

        self.context.set_filter("none");
        self.context.set_global_alpha(1.0);
    }
}
