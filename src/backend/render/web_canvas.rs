use super::RenderBackend;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement>;

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

impl WebCanvas {
    fn new(context: CanvasRenderingContext2d, width: f64, height: f64) -> WebCanvas {
        context: CanvasRenderingContext2d,
        width: f64,
        height: f64,
    }
}

impl RenderBackend for WebCanvas {
    fn register_shape(&mut self, shape: &swf::Shape) -> ShapeHandle {
        let handle = ShapeHandle(self.meshes.len());

        let image = HtmlImageElement::new().unwrap();
        
        use url::percent_encoding::{percent_encode, DEFAULT_ENCODE_SET};
        let svg = super::shape_utils::swf_shape_to_paths(&shape);
        let svg_encoded = format!("data:image/xvg+xml;{}", percent_encode(svg, DEFAULT_ENCODE_SET));
        image.set_src(&svg_encoded);

        self.shapes.push(ShapeData{
            image, x_min: shape.shape_bounds.x_min.into(), y_min: shape.shape_bounds.y_min.into()
        });

        handle
    }

    fn begin_frame(&mut self) {
        context.reset_transform();
    }

    fn end_frame(&mut self) {
        // Noop
    }

    fn clear(&mut self, color: Color) {
        let color = format!("rgb({}, {}, {}", color.r, color.g, color.b);
        context.fill_rect(0, 0, self.width, self.height, &color);
    }

    fn render_shape(&mut self, shape: ShapeHandle, matrix: &Matrix) {
        let shape = &self.shapes[shape.0];
        context.set_transform(matrix.a, matrix.b, matrix.c, matrix.d, matrix.tx, matrix.ty).unwrap();
        context.draw_image_with_html_image_element(&shape.image, shape.x_min, shape.y_min).unwrap();
    }
}