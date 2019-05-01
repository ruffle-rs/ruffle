use super::{common::ShapeHandle, RenderBackend};
use crate::{transform::Transform, Color};
use log::info;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, HtmlImageElement};

pub struct WebCanvasRenderBackend {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    color_matrix: Element,
    shapes: Vec<ShapeData>,
}

struct ShapeData {
    image: HtmlImageElement,
    x_min: f64,
    y_min: f64,
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

        info!("{:?}", svg);

        svg.set_attribute("width", "0");
        svg.set_attribute("height", "0");
        svg.set_attribute_ns(
            Some("http://www.w3.org/2000/xmlns/"),
            "xmlns:xlink",
            "http://www.w3.org/1999/xlink",
        )
        .map_err(|_| "Couldn't make SVG")?;

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
        svg.append_child(&filter)
            .map_err(|_| "append_child failed")?;
        let body = document
            .body()
            .unwrap()
            .append_child(&svg)
            .map_err(|_| "append_child failed")?;

        Ok(Self {
            canvas: canvas.clone(),
            color_matrix,
            context,
            shapes: vec![],
        })
    }
}

impl RenderBackend for WebCanvasRenderBackend {
    fn set_dimensions(&mut self, _width: u32, _height: u32) {}

    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.shapes.len());

        let image = HtmlImageElement::new().unwrap();

        use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
        let svg = super::shape_utils::swf_shape_to_svg(&shape);
        let svg_encoded = format!(
            "data:image/svg+xml,{}",
            utf8_percent_encode(&svg, DEFAULT_ENCODE_SET)
        );
        image.set_src(&svg_encoded);

        self.shapes.push(ShapeData {
            image,
            x_min: shape.shape_bounds.x_min.into(),
            y_min: shape.shape_bounds.y_min.into(),
        });

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

    fn render_shape(&mut self, shape: ShapeHandle, transform: &Transform) {
        let shape = &self.shapes[shape.0];
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
