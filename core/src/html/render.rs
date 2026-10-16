//! Rendering shared by text fields and FTE text lines.

use crate::context::RenderContext;
use crate::font::{EvalParameters, FontLike, FontSet};
use crate::string::WStr;
use ruffle_render::transform::Transform;
use std::ops::Range;
use swf::{ColorTransform, Twips};

/// Render a run of glyphs at the current transform and return the caret's x position.
///
/// Selection and caret positions are relative to `text`. The caller is responsible
/// for positioning and clipping the run, and for drawing the selection background,
/// caret, and underline.
pub fn render_text<'gc>(
    context: &mut RenderContext<'_, 'gc>,
    text: &WStr,
    font: FontSet<'gc>,
    params: EvalParameters,
    selection: Option<Range<usize>>,
    caret: Option<usize>,
) -> Twips {
    let mut caret_x = Twips::ZERO;
    font.evaluate(text, params, |pos, transform, glyph, advance, x| {
        if glyph.renderable(context) {
            // Selected glyphs are white, independently of the text color.
            if selection.as_ref().is_some_and(|range| range.contains(&pos)) {
                context.transform_stack.push(&Transform {
                    matrix: transform.matrix,
                    color_transform: ColorTransform::IDENTITY,
                    perspective_projection: transform.perspective_projection,
                    tz: transform.tz,
                });
            } else {
                context.transform_stack.push(transform);
            }
            glyph.render(context);
            context.transform_stack.pop();
        }

        if let Some(caret) = caret {
            if pos == caret {
                caret_x = x;
            } else if caret > 0 && pos == caret - 1 {
                // The caret may be rendered at the end, after all glyphs.
                caret_x = x + advance;
            }
        }
    });
    caret_x
}
