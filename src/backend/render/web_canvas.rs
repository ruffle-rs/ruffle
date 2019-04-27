use super::{common::ShapeHandle, RenderBackend};
use crate::{Color, matrix::Matrix};
use log::info;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

pub struct WebCanvasRenderBackend {
    context: CanvasRenderingContext2d,
    width: f64,
    height: f64,

    shapes: Vec<ShapeData>,
}

struct ShapeData {
    image: HtmlImageElement,
    x_min: f64,
    y_min: f64,
}

impl WebCanvasRenderBackend {
    pub fn new(context: CanvasRenderingContext2d, width: f64, height: f64) -> Self {
        Self {
            context,
            width,
            height,
            shapes: vec![],
        }
    }
}

impl RenderBackend for WebCanvasRenderBackend {
    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.shapes.len());

        let image = HtmlImageElement::new().unwrap();
        
        use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
        let svg = super::shape_utils::swf_shape_to_svg(&shape);
        let svg_encoded = format!("data:image/svg+xml,{}", utf8_percent_encode(&svg, DEFAULT_ENCODE_SET));
        info!("{}", svg_encoded);
        image.set_src(&svg_encoded);

        self.shapes.push(ShapeData{
            image, x_min: shape.shape_bounds.x_min.into(), y_min: shape.shape_bounds.y_min.into()
        });

        handle
    }

    fn begin_frame(&mut self) {
        self.context.reset_transform();
    }

    fn end_frame(&mut self) {
        // Noop
    }

    fn clear(&mut self, color: Color) {
        let color = format!("rgb({}, {}, {}", color.r, color.g, color.b);
        self.context.set_fill_style(&color.into());
        self.context.fill_rect(0.0, 0.0, self.width, self.height);
    }

    fn render_shape(&mut self, shape: ShapeHandle, matrix: &Matrix) {
        let shape = &self.shapes[shape.0];
        self.context.set_transform(matrix.a.into(), matrix.b.into(), matrix.c.into(), matrix.d.into(), matrix.tx.into(), matrix.ty.into()).unwrap();
        self.context.draw_image_with_html_image_element(&shape.image, shape.x_min, shape.y_min).unwrap();
    }
}