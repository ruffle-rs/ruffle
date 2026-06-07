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
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub char_start: usize,
    pub char_end: usize,
    pub x: f32,
    pub width: f32,
    pub bidi_level: u8,
    pub word_boundary_on_left: bool,
}

#[derive(Collect)]
#[collect(no_drop)]
pub struct TextLineLayout<'gc> {
    html_line: LayoutLine<'gc>,
    #[collect(require_static)]
    text: WString,
    #[collect(require_static)]
    atoms: Vec<Atom>,
    #[collect(require_static)]
    ascent: f32,
    #[collect(require_static)]
    descent: f32,
}

impl<'gc> TextLineLayout<'gc> {
    pub fn new(
        html_line: LayoutLine<'gc>,
        text: WString,
        text_block_begin: usize,
        bidi_level: u8,
    ) -> Self {
        let atoms = build_atoms(&html_line, &text, text_block_begin, bidi_level);
        let (ascent, descent) = typo_metrics(&html_line, &text);
        Self {
            html_line,
            text,
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
        if self.is_tab_only_line() {
            return self.atoms.last().map_or(0.0, |atom| atom.x + atom.width);
        }
        let start = self.html_line.text_range().start;
        let chars: Vec<u16> = self.text.iter().collect();
        for (i, atom) in self.atoms.iter().enumerate().rev() {
            if !is_blank_unit(chars.get(start + i).copied()) {
                return atom.x + atom.width;
            }
        }
        0.0
    }

    pub fn width(&self) -> f32 {
        if self.is_tab_only_line() {
            return self.text_width();
        }
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
        if self.is_tab_only_line() {
            return 1;
        }
        let mut len = range.len();
        if self
            .text
            .get(range.end)
            .is_some_and(|unit| matches!(unit, 0x2028 | 0x2029))
            && range.end + 1 == self.text.len()
        {
            len += 1;
        }
        if range.end > range.start
            && self.text.get(range.end - 1) == Some(0x0d)
            && self.text.get(range.end) == Some(0x0a)
        {
            len += 1;
        }
        len
    }

    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }

    fn is_tab_only_line(&self) -> bool {
        let range = self.html_line.text_range();
        range.is_empty() && self.text.iter().any(|unit| unit == 0x09)
    }
}

fn build_atoms(
    line: &LayoutLine<'_>,
    text: &WStr,
    text_block_begin: usize,
    bidi_level: u8,
) -> Vec<Atom> {
    let range = line.text_range();
    let line_width = line.bounds().width().to_pixels() as f32;
    let word_bounds = word_boundary_offsets(text);

    let mut kern_after = vec![0i32; range.len()];
    for lbox in line.boxes_iter() {
        let Some((box_text, _tf, font, params, _color)) = lbox.as_renderable_text(text) else {
            continue;
        };
        if !params.kerning || !font.has_kerning_info() {
            continue;
        }

        let units: Vec<u16> = box_text.iter().collect();
        for k in 0..units.len().saturating_sub(1) {
            let left = char::from_u32(units[k] as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
            let right = char::from_u32(units[k + 1] as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
            let twips = font.pair_kerning(params, left, right).get();
            let Some(i) = (lbox.start() + k).checked_sub(range.start) else {
                continue;
            };
            if let Some(slot) = kern_after.get_mut(i) {
                *slot = twips;
            }
        }
    }

    let consumes_trailing_hard_break = text
        .get(range.end)
        .is_some_and(|unit| matches!(unit, 0x2028 | 0x2029))
        && range.end + 1 == text.len();
    if range.is_empty() {
        let Some(pos) = text.iter().position(|unit| unit == 0x09) else {
            return Vec::new();
        };
        return vec![Atom {
            char_start: text_block_begin + pos,
            char_end: text_block_begin + pos + 1,
            x: 0.0,
            width: 48.0,
            bidi_level,
            word_boundary_on_left: true,
        }];
    }
    let mut atoms = Vec::with_capacity(range.len() + usize::from(consumes_trailing_hard_break));
    for (i, pos) in range.clone().enumerate() {
        let (x, width) = match line.char_bounds(pos) {
            Some(rect) => {
                let kern_in = if i > 0 { kern_after[i - 1] } else { 0 };
                let kern_out = kern_after[i];
                let x = rect.x_min - Twips::new(kern_in / 2);
                let x_max = rect.x_max - Twips::new(kern_out / 2);
                (x.to_pixels() as f32, (x_max - x).to_pixels() as f32)
            }
            None => (line_width, 0.0),
        };
        atoms.push(Atom {
            char_start: text_block_begin + pos,
            char_end: text_block_begin + pos + 1,
            x,
            width,
            bidi_level,
            word_boundary_on_left: pos == range.start || word_bounds.binary_search(&pos).is_ok(),
        });
    }
    if consumes_trailing_hard_break {
        atoms.push(Atom {
            char_start: text_block_begin + range.end,
            char_end: text_block_begin + range.end + 1,
            x: line_width,
            width: 0.0,
            bidi_level: 0,
            word_boundary_on_left: range.end == range.start
                || word_bounds.binary_search(&range.end).is_ok(),
        });
    }
    atoms
}

fn typo_metrics(line: &LayoutLine<'_>, text: &WStr) -> (f32, f32) {
    let mut ascent = 0.0_f32;
    let mut descent = 0.0_f32;
    let mut found = false;
    let blank_line = line.text_range().all(|pos| is_blank_unit(text.get(pos)));

    for lbox in line.boxes_iter() {
        if let Some((_, _, font_set, params, _)) = lbox.as_renderable_text(text) {
            if blank_line {
                let height = params.height().to_pixels() as f32;
                ascent = ascent.max(height * (389.0 / 512.0));
                descent = descent.max(height * (123.0 / 512.0));
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
        Some(unit) => matches!(unit, 0x20 | 0x0a | 0x0d | 0x2028 | 0x2029),
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
