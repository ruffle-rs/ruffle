//! The FteTextLine DisplayObject, the backing for flash.text.engine.TextLine.
//!
//! A TextLine holds one laid-out line of text. Layout runs through Ruffle's
//! shared engine (crate::html::lower_from_text_spans), the same engine that
//! backs EditText, so FTE text and TextField text shape identically and
//! share device and embedded font resolution.
//!
//! TextLine in AS3 extends DisplayObjectContainer, and so InteractiveObject,
//! so the backing implements TInteractiveObject.

use crate::avm2::StageObject as Avm2StageObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{Avm2MousePick, BoundsMode, DisplayObjectBase, InteractiveObject};
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
use ruffle_render::transform::Transform;
use std::cell::Ref;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

/// One FTE atom: a grapheme cluster, the unit TextLine's atom API operates
/// on. Atoms tile the line's entire text range, including the zero-width
/// paragraph terminator that owns no glyph, so any in-range char index resolves
/// to exactly one atom.
#[derive(Clone, Copy, Debug)]
pub struct Atom {
    /// First char index covered (UTF-16, text-block-absolute).
    pub char_start: usize,
    /// One past the last char index covered.
    pub char_end: usize,
    /// X position relative to the line start, in pixels.
    pub x: f32,
    /// Atom advance width in pixels.
    pub width: f32,
    pub bidi_level: u8,
    /// Whether a UAX#29 word boundary sits immediately left of this atom
    /// (the line start always counts).
    pub word_boundary_on_left: bool,
}

/// The laid-out content of one FTE TextLine: an html layout line plus the
/// derived atom list and the source text needed to render it.
#[derive(Collect)]
#[collect(no_drop)]
pub struct FteLine<'gc> {
    /// The shared-engine layout line (one line of glyphs + boxes).
    html_line: LayoutLine<'gc>,
    /// The displayed text the layout was built from. It is a TextBlock's
    /// content from textBlockBeginIndex onward; LayoutBoxes index into it.
    #[collect(require_static)]
    text: WString,
    /// Atoms, in logical order, with text-block-absolute char indices.
    #[collect(require_static)]
    atoms: Vec<Atom>,
    /// Line ascent and descent in pixels, from the font's OS/2 typographic
    /// metrics. These are the values flash.text.engine reports for
    /// TextLine.ascent and descent, which drive TLF line spacing. The
    /// shared layout engine's own LayoutLine metrics are hhea-derived and
    /// used for glyph layout.
    #[collect(require_static)]
    ascent: f32,
    #[collect(require_static)]
    descent: f32,
}

impl<'gc> FteLine<'gc> {
    /// Build an FteLine from a shared-engine layout line.
    ///
    /// text is the displayed text that was laid out (the TextBlock content
    /// from text_block_begin onward). text_block_begin is added to every
    /// atom's char index so the FTE API speaks text-block-absolute coordinates.
    pub fn new(html_line: LayoutLine<'gc>, text: WString, text_block_begin: usize) -> Self {
        let atoms = build_atoms(&html_line, &text, text_block_begin);
        let (ascent, descent) = typo_metrics(&html_line, &text);
        Self {
            html_line,
            text,
            atoms,
            ascent,
            descent,
        }
    }

    /// Line ascent in pixels. This is the OS/2 typographic metric; see the
    /// struct field.
    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    /// Line descent in pixels (the OS/2 typographic metric).
    pub fn descent(&self) -> f32 {
        self.descent
    }

    /// Total advance width of the line in pixels, trailing whitespace
    /// included. This is the rendered and hit-test width.
    pub fn width(&self) -> f32 {
        self.html_line.bounds().width().to_pixels() as f32
    }

    /// Advance width excluding trailing whitespace, in pixels. This is the
    /// value flash.text.engine.TextLine.textWidth reports. Trailing spaces
    /// overhang the line in FTE and are not counted; a line that is all
    /// whitespace has a textWidth of 0.
    pub fn text_width(&self) -> f32 {
        let start = self.html_line.text_range().start;
        let chars: Vec<u16> = self.text.iter().collect();
        for (i, atom) in self.atoms.iter().enumerate().rev() {
            let blank = match chars.get(start + i) {
                Some(&c) => matches!(c, 0x20 | 0x09 | 0x0a | 0x0d | 0x2028 | 0x2029),
                None => true,
            };
            if !blank {
                return atom.x + atom.width;
            }
        }
        0.0
    }

    /// Number of UTF-16 code units this line consumes from the text block.
    pub fn raw_text_length(&self) -> usize {
        self.html_line.text_range().len()
    }

    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
}

/// Derive the atom list for a layout line.
///
/// Atoms tile [line.start, line.end) one char at a time. A grapheme cluster
/// is one atom; for the Latin text the FTE engine handles, that is one UTF-16
/// unit. A position with no glyph, notably the \u{2029} terminator, becomes
/// a zero-width atom positioned at the line end.
fn build_atoms(line: &LayoutLine<'_>, text: &WStr, text_block_begin: usize) -> Vec<Atom> {
    let range = line.text_range();
    let line_width = line.bounds().width().to_pixels() as f32;

    // UAX#29 word-boundary char offsets, relative to text.
    let word_bounds = word_boundary_offsets(text);

    // Per-position pair kerning, in twips: kern_after[i] is the kern baked
    // into the advance of the char at range.start + i.
    //
    // Flash Player quirk: its FTE atom model splits a pair-kern half and half
    // between the two atoms it sits between. getAtomBounds reports the left
    // atom ending, and the right atom starting, at the midpoint of the kern
    // gap, even though the glyphs are drawn at the full kerned pen position
    // and textWidth still reflects the full kern. Replicate the split here
    // so atom bounds match Flash. The line's total width is unaffected, since
    // each kern contributes half to one atom and half to the next.
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

    let mut atoms = Vec::with_capacity(range.len());
    for (i, pos) in range.clone().enumerate() {
        let (x, width) = match line.char_bounds(pos) {
            Some(rect) => {
                // Pull each atom edge back to the midpoint of its kern gap.
                let kern_in = if i > 0 { kern_after[i - 1] } else { 0 };
                let kern_out = kern_after[i];
                let x = rect.x_min - Twips::new(kern_in / 2);
                let x_max = rect.x_max - Twips::new(kern_out / 2);
                (x.to_pixels() as f32, (x_max - x).to_pixels() as f32)
            }
            None => (line_width, 0.0),
        };
        let first_in_line = pos == range.start;
        atoms.push(Atom {
            char_start: text_block_begin + pos,
            char_end: text_block_begin + pos + 1,
            x,
            width,
            bidi_level: 0,
            word_boundary_on_left: first_in_line || word_bounds.binary_search(&pos).is_ok(),
        });
    }
    atoms
}

/// A line's ascent and descent in pixels, from the OS/2 typographic metrics
/// of its fonts. These are the values FTE reports for TextLine.ascent and
/// descent. Flash's flash.text.engine uses the OS/2 sTypo* metrics here,
/// not the hhea ascender and descender that the shared layout engine uses
/// for glyph placement. The two differ for many fonts (Arial and Liberation
/// Sans by about 24%), and TLF's line-spacing math reads these values, so
/// getting them right is what makes FTE line spacing match Flash.
fn typo_metrics(line: &LayoutLine<'_>, text: &WStr) -> (f32, f32) {
    let mut ascent = 0.0_f32;
    let mut descent = 0.0_f32;
    let mut found = false;
    for lbox in line.boxes_iter() {
        if let Some((_, _, font_set, params, _)) = lbox.as_renderable_text(text) {
            let font = font_set.main_font();
            ascent = ascent.max(font.typo_ascent(params.height()).to_pixels() as f32);
            descent = descent.max(font.typo_descent(params.height()).to_pixels() as f32);
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

/// Sorted UAX#29 word-boundary offsets (in UTF-16 code units) of text.
fn word_boundary_offsets(text: &WStr) -> Vec<usize> {
    let utf8 = text.to_utf8_lossy();
    // prefix[b] = UTF-16 code units in utf8[..b], for every byte boundary.
    let mut prefix = vec![0usize; utf8.len() + 1];
    let mut utf16 = 0usize;
    let mut prev = 0usize;
    for (b, c) in utf8.char_indices() {
        for slot in prefix.iter_mut().take(b + 1).skip(prev) {
            *slot = utf16;
        }
        utf16 += c.len_utf16();
        prev = b + c.len_utf8();
    }
    for slot in prefix.iter_mut().skip(prev) {
        *slot = utf16;
    }

    let mut bounds: Vec<usize> = utf8
        .split_word_bound_indices()
        .map(|(b, _)| prefix[b])
        .collect();
    bounds.dedup();
    bounds
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct FteTextLine<'gc>(Gc<'gc, FteTextLineData<'gc>>);

impl fmt::Debug for FteTextLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FteTextLine")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct FteTextLineData<'gc> {
    base: InteractiveObjectBase<'gc>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
    line: RefLock<FteLine<'gc>>,
    #[collect(require_static)]
    movie: Arc<SwfMovie>,
}

impl<'gc> FteTextLine<'gc> {
    pub fn new(context: &mut UpdateContext<'gc>, movie: Arc<SwfMovie>, line: FteLine<'gc>) -> Self {
        FteTextLine(Gc::new(
            context.gc(),
            FteTextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                line: RefLock::new(line),
                movie,
            },
        ))
    }

    /// Borrow the cached FteLine.
    pub fn line(self) -> Ref<'gc, FteLine<'gc>> {
        Gc::as_ref(self.0).line.borrow()
    }

    /// Replace the cached line. TextBlock.recreateTextLine uses this to
    /// re-lay-out an existing TextLine in place.
    pub fn set_line(self, context: &mut UpdateContext<'gc>, line: FteLine<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), FteTextLineData, line).replace(line);
    }
}

impl<'gc> TDisplayObject<'gc> for FteTextLine<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        let interactive: Gc<'gc, InteractiveObjectBase<'gc>> = HasPrefixField::as_prefix_gc(self.0);
        HasPrefixField::as_prefix_gc(interactive)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        let borrowed = self.0.line.borrow();
        let cloned = FteTextLineData {
            base: Default::default(),
            avm2_object: Lock::new(None),
            line: RefLock::new(FteLine {
                html_line: borrowed.html_line.clone(),
                text: borrowed.text.clone(),
                atoms: borrowed.atoms.clone(),
                ascent: borrowed.ascent,
                descent: borrowed.descent,
            }),
            movie: self.0.movie.clone(),
        };
        drop(borrowed);
        Self(Gc::new(gc_context, cloned)).into()
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

        // The shared layout engine lays a line out in a top-origin space, with
        // y=0 at the line's top and the baseline ascent below that. A
        // flash.text.engine.TextLine is baseline-origin: y=0 is the Roman
        // baseline, the way self_bounds reports it. Shift the whole line up
        // by its baseline before drawing the layout boxes; otherwise every
        // line renders ascent too low, leaving a large gap above the first.
        let baseline = line.html_line.bounds().origin().y() + line.html_line.ascent();
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(Twips::ZERO, -baseline),
            ..Default::default()
        });

        for lbox in line.html_line.boxes_iter() {
            let origin = lbox.bounds().origin();
            let renderable = lbox.as_renderable_text(&line.text);
            // ElementFormat.baselineShift is a per-run vertical offset of the
            // glyphs. In flash.text.engine a positive value shifts the run
            // down (toward larger y) and a negative value up, which is the
            // opposite of the CSS baseline-shift convention. It moves only
            // where the run is drawn; line metrics and atom bounds do not
            // change, so it is applied here in the render transform.
            let mut baseline_shift = Twips::ZERO;
            if let Some((_, tf, ..)) = &renderable
                && let Some(shift) = tf.baseline_shift
            {
                baseline_shift = Twips::from_pixels(shift);
            }
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
        unlock!(Gc::write(mc, self.0), FteTextLineData, avm2_object).set(Some(to));
    }
}

impl<'gc> TInteractiveObject<'gc> for FteTextLine<'gc> {
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
