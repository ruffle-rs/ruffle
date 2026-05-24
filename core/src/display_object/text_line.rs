//! The TextLine DisplayObject, backing flash.text.engine TextLine.

use crate::avm2::StageObject as Avm2StageObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{
    Avm2MousePick, BoundsMode, DisplayObjectBase, EditText, InteractiveObject,
};
use crate::events::{ClipEvent, ClipEventResult};
use crate::font::FontLike;
use crate::html::LayoutLine;
use crate::prelude::*;
use crate::string::{WStr, WString};
use crate::tag_utils::SwfMovie;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::cell::Ref;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub char_start: usize,
    pub char_end: usize,
    pub x: f32,
    pub width: f32,
    pub word_boundary_on_left: bool,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct TextLineLayout<'gc> {
    html_line: LayoutLine<'gc>,
    #[collect(require_static)]
    text: WString,
    text_block_begin: usize,
    #[collect(require_static)]
    atoms: Vec<Atom>,
    #[collect(require_static)]
    ascent: f32,
    #[collect(require_static)]
    descent: f32,
}

impl<'gc> TextLineLayout<'gc> {
    pub fn new(html_line: LayoutLine<'gc>, text: WString, text_block_begin: usize) -> Self {
        let (ascent, descent) = typo_metrics(&html_line, &text);
        let range = html_line.text_range();
        let consumes_trailing_hard_break = text
            .get(range.end)
            .is_some_and(|unit| matches!(unit, 0x2028 | 0x2029))
            && range.end + 1 == text.len();
        let mut atoms: Vec<Atom> = range
            .clone()
            .map(|pos| Atom {
                char_start: text_block_begin + pos,
                char_end: text_block_begin + pos + 1,
                x: html_line
                    .char_bounds(pos)
                    .map(|bounds| bounds.x_min.to_pixels() as f32)
                    .unwrap_or_else(|| html_line.bounds().width().to_pixels() as f32),
                width: html_line
                    .char_bounds(pos)
                    .map(|bounds| bounds.width().to_pixels() as f32)
                    .unwrap_or(0.0),
                word_boundary_on_left: pos == range.start
                    || text
                        .iter()
                        .nth(pos)
                        .map(|c| matches!(c, 0x20 | 0x09 | 0x0a | 0x0d))
                        .unwrap_or(false),
            })
            .collect();
        if consumes_trailing_hard_break {
            atoms.push(Atom {
                char_start: text_block_begin + range.end,
                char_end: text_block_begin + range.end + 1,
                x: html_line.bounds().width().to_pixels() as f32,
                width: 0.0,
                word_boundary_on_left: range.end == range.start
                    || text
                        .iter()
                        .nth(range.end)
                        .map(|c| matches!(c, 0x20 | 0x09 | 0x0a | 0x0d))
                        .unwrap_or(false),
            });
        }
        Self {
            html_line,
            text,
            text_block_begin,
            atoms,
            ascent,
            descent,
        }
    }

    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    pub fn descent(&self) -> f32 {
        self.descent
    }

    pub fn text_width(&self) -> f32 {
        let chars: Vec<u16> = self.text.iter().collect();
        for atom in self.atoms.iter().rev() {
            let pos = atom.char_start.saturating_sub(self.text_block_begin);
            if !is_blank_unit(chars.get(pos).copied()) {
                return atom.x + atom.width;
            }
        }
        0.0
    }

    pub fn width(&self) -> f32 {
        if self
            .html_line
            .text_range()
            .all(|pos| is_blank_unit(self.text.get(pos)))
        {
            return 0.0;
        }

        self.html_line.bounds().width().to_pixels() as f32
    }

    pub fn raw_text_length(&self) -> usize {
        let range = self.html_line.text_range();
        let mut len = range.len();
        if self
            .text
            .get(range.end)
            .is_some_and(|unit| matches!(unit, 0x2028 | 0x2029))
            && range.end + 1 == self.text.len()
        {
            len += 1;
        }
        len
    }

    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
}

fn typo_metrics(line: &LayoutLine<'_>, text: &WStr) -> (f32, f32) {
    let mut ascent = 0.0_f32;
    let mut descent = 0.0_f32;
    let mut found = false;
    let blank_line = line
        .text_range()
        .all(|pos| is_blank_unit(text.get(pos)));

    for lbox in line.boxes_iter() {
        if let Some((_, _, font_set, params, _)) = lbox.as_renderable_text(text) {
            if blank_line {
                let height = params.height().to_pixels() as f32;
                ascent = ascent.max(height * 0.76);
                descent = descent.max(height * 0.24);
            } else {
                let font = font_set.main_font();
                ascent = ascent.max(font.typo_ascent(params.height()).to_pixels() as f32);
                descent = descent.max(font.typo_descent(params.height()).to_pixels() as f32);
            }
            found = true;
        }
    }

    if found {
        (ascent, descent)
    } else {
        (
            line.ascent().to_pixels() as f32,
            line.descent().to_pixels() as f32,
        )
    }
}

fn is_blank_unit(unit: Option<u16>) -> bool {
    match unit {
        Some(unit) => matches!(unit, 0x20 | 0x09 | 0x0a | 0x0d | 0x2028 | 0x2029),
        None => true,
    }
}

fn word_boundary_offsets(text: &WStr) -> Vec<usize> {
    let utf8 = text.to_utf8_lossy();
    let mut prefix = vec![0usize; utf8.len() + 1];
    let mut utf16 = 0usize;
    let mut prev = 0usize;

    for (byte, ch) in utf8.char_indices() {
        for slot in prefix.iter_mut().take(byte + 1).skip(prev) {
            *slot = utf16;
        }
        utf16 += ch.len_utf16();
        prev = byte + ch.len_utf8();
    }
    for slot in prefix.iter_mut().skip(prev) {
        *slot = utf16;
    }

    let mut bounds = Vec::new();
    for (byte, _) in utf8.split_word_bound_indices() {
        bounds.push(prefix[byte]);
    }
    bounds.dedup();
    bounds
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct TextLine<'gc>(Gc<'gc, TextLineData<'gc>>);

impl fmt::Debug for TextLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextLine")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct TextLineData<'gc> {
    base: InteractiveObjectBase<'gc>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
    line: RefLock<TextLineLayout<'gc>>,
    fallback: RefLock<EditText<'gc>>,
    #[collect(require_static)]
    movie: Arc<SwfMovie>,
}

impl<'gc> TextLine<'gc> {
    pub fn new(
        context: &mut UpdateContext<'gc>,
        movie: Arc<SwfMovie>,
        line: TextLineLayout<'gc>,
        fallback: EditText<'gc>,
    ) -> Self {
        TextLine(Gc::new(
            context.gc(),
            TextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                line: RefLock::new(line),
                fallback: RefLock::new(fallback),
                movie,
            },
        ))
    }

    pub fn line(self) -> Ref<'gc, TextLineLayout<'gc>> {
        Gc::as_ref(self.0).line.borrow()
    }

    pub fn set_line(
        self,
        context: &mut UpdateContext<'gc>,
        line: TextLineLayout<'gc>,
        fallback: EditText<'gc>,
    ) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), TextLineData, line).replace(line);
        *unlock!(Gc::write(mc, self.0), TextLineData, fallback).borrow_mut() = fallback;
    }
}

impl<'gc> TDisplayObject<'gc> for TextLine<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        let interactive: Gc<'gc, InteractiveObjectBase<'gc>> = HasPrefixField::as_prefix_gc(self.0);
        HasPrefixField::as_prefix_gc(interactive)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        let borrowed = self.0.line.borrow();
        Self(Gc::new(
            gc_context,
            TextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                line: RefLock::new(TextLineLayout {
                    html_line: borrowed.html_line.clone(),
                    text: borrowed.text.clone(),
                    text_block_begin: borrowed.text_block_begin,
                    atoms: borrowed.atoms.clone(),
                    ascent: borrowed.ascent,
                    descent: borrowed.descent,
                }),
                fallback: RefLock::new(*self.0.fallback.borrow()),
                movie: self.0.movie.clone(),
            },
        ))
        .into()
    }

    fn id(self) -> CharacterId {
        0
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
    }

    fn replace_with(self, _context: &mut UpdateContext<'gc>, _id: CharacterId) {}

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        let line = self.0.line.borrow();
        let mut has_renderable_content = false;
        for lbox in line.html_line.boxes_iter() {
            if lbox.as_renderable_drawing().is_some() {
                has_renderable_content = true;
                break;
            }
            if let Some((text, _tf, font, params, _color)) = lbox.as_renderable_text(&line.text) {
                font.evaluate(
                    text,
                    Default::default(),
                    params,
                    &mut |_pos, _glyph_transform, glyph, _advance, _x| {
                        has_renderable_content |= glyph.renderable(context);
                    },
                );
                if has_renderable_content {
                    break;
                }
            }
        }
        if !has_renderable_content {
            self.0.fallback.borrow().render_self(context);
            return;
        }

        let baseline = line.html_line.bounds().origin().y() + line.html_line.ascent();
        let (quality_offset, low_quality_line_offset) =
            if context.stage.quality() == StageQuality::Low {
                (Twips::from_pixels(2.0), Twips::new(15))
            } else {
                (Twips::ZERO, Twips::ZERO)
            };
        let line_offset = if line.atoms.first().is_some_and(|atom| atom.char_start > 0) {
            low_quality_line_offset
        } else {
            Twips::ZERO
        };
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(quality_offset, quality_offset + line_offset - baseline),
            ..Default::default()
        });

        for lbox in line.html_line.boxes_iter() {
            let origin = lbox.bounds().origin();
            let renderable = lbox.as_renderable_text(&line.text);
            let baseline_shift = match renderable.as_ref() {
                Some((_, tf, ..)) => match tf.baseline_shift {
                    Some(shift) => Twips::from_pixels(shift),
                    None => Twips::ZERO,
                },
                None => Twips::ZERO,
            };
            context.transform_stack.push(&Transform {
                matrix: Matrix::translate(origin.x(), origin.y() + baseline_shift),
                ..Default::default()
            });

            if let Some((text, _tf, font, params, color)) = renderable {
                let mut transform: Transform = Default::default();
                transform.color_transform.set_mult_color(color);
                font.evaluate(
                    text,
                    transform,
                    params,
                    &mut |_pos, glyph_transform, glyph, _advance, _x| {
                        if glyph.renderable(context) {
                            context.transform_stack.push(glyph_transform);
                            glyph.render(context);
                            context.transform_stack.pop();
                        }
                    },
                );
            }

            if let Some(drawing) = lbox.as_renderable_drawing() {
                drawing.render(context);
            }

            context.transform_stack.pop();
        }

        context.transform_stack.pop();
    }

    fn self_bounds(self, _mode: BoundsMode) -> Rectangle<Twips> {
        let line = self.0.line.borrow();
        Rectangle {
            x_min: Twips::ZERO,
            x_max: Twips::from_pixels(line.width() as f64),
            y_min: Twips::from_pixels(-(line.ascent() as f64)),
            y_max: Twips::from_pixels(line.descent() as f64),
        }
    }

    fn hit_test_shape(
        self,
        _context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        options: HitTestOptions,
    ) -> bool {
        if options.contains(HitTestOptions::SKIP_INVISIBLE) && !self.visible() {
            return false;
        }
        self.world_bounds(BoundsMode::Engine).contains(point)
    }

    fn object1(self) -> Option<crate::avm1::Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), TextLineData, avm2_object).set(Some(to));
    }
}

impl<'gc> TInteractiveObject<'gc> for TextLine<'gc> {
    fn raw_interactive(self) -> Gc<'gc, InteractiveObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn event_dispatch(
        self,
        _context: &mut UpdateContext<'gc>,
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn mouse_pick_avm1(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        None
    }

    fn mouse_pick_avm2(
        self,
        _context: &mut UpdateContext<'gc>,
        _point: Point<Twips>,
        _require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        Avm2MousePick::Miss
    }

    fn mouse_cursor(self, _context: &mut UpdateContext<'gc>) -> MouseCursor {
        MouseCursor::Arrow
    }
}
