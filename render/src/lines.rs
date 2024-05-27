use crate::commands::CommandHandler;
use crate::matrix::Matrix;
use swf::{Color, Point, PointDelta, Twips};

/// Draw a line using [`CommandHandler::draw_rect()`].
///
/// This is a universal way of drawing a line, independent of
/// the rendering backend's implementation and rasterization rules.
pub fn emulate_line(handler: &mut impl CommandHandler, color: Color, matrix: Matrix) {
    let a = matrix * Point::new(Twips::ZERO, Twips::ZERO);
    let b = matrix * Point::new(Twips::ONE, Twips::ZERO);
    emulate_line_as_rect(handler, color, a, b);
}

/// Similar to [`emulate_line`], but emulates drawing a rectangle with lines as its sides.
pub fn emulate_line_rect(handler: &mut impl CommandHandler, color: Color, matrix: Matrix) {
    let a = matrix * Point::new(Twips::ZERO, Twips::ZERO);
    let b = matrix * Point::new(Twips::ONE, Twips::ZERO);
    let c = matrix * Point::new(Twips::ONE, Twips::ONE);
    let d = matrix * Point::new(Twips::ZERO, Twips::ONE);
    emulate_line_as_rect(handler, color, a, b);
    emulate_line_as_rect(handler, color, b, c);
    emulate_line_as_rect(handler, color, c, d);
    emulate_line_as_rect(handler, color, d, a);
}

// Note that we cannot simply crate a rect and then transform it using
// the transformation matrix, as the thickness and line caps should not be transformed.
// That's why two already transformed [`Point`]s are taken as input,
// and then a 1px wide rectangle is fit inbetween them.
fn emulate_line_as_rect(
    handler: &mut impl CommandHandler,
    color: Color,
    a: Point<Twips>,
    b: Point<Twips>,
) {
    let PointDelta { dx, dy } = b - a;
    let (dx, dy) = (dx.to_pixels(), dy.to_pixels());

    let len = (dx * dx + dy * dy).sqrt() as f32;
    let angle = dy.atan2(dx) as f32;

    let rotation = Matrix::rotate(angle);
    let translation = Matrix::translate(a.x + Twips::HALF, a.y + Twips::HALF);

    // Step 1: Create a 1px thick line with the proper length.
    let line = Matrix::create_box(len, 1.0, Twips::ZERO, -Twips::HALF);
    // Step 2: Rotate it, so it still starts at `(0,0)` but points to `b-a`.
    let line = rotation * line;
    // Step 3: Translate it to `a`, so that it points to `b`.
    let line = translation * line;
    handler.draw_rect(color, line);
}
