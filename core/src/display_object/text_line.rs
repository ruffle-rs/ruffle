//! The TextLine DisplayObject, backing flash.text.engine TextLine.

use crate::avm2::StageObject as Avm2StageObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{
    Avm2MousePick, BoundsMode, DisplayObjectBase, EditText, InteractiveObject,
};
use crate::events::{ClipEvent, ClipEventResult};
use crate::html::LayoutLine;
use crate::prelude::*;
use crate::string::WString;
use crate::tag_utils::SwfMovie;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::utils::HasPrefixField;
use std::cell::Ref;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub char_start: usize,
    pub char_end: usize,
    pub x: f32,
    pub width: f32,
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
    pub fn new(html_line: LayoutLine<'gc>, text: WString, text_block_begin: usize) -> Self {
        let ascent = html_line.ascent().to_pixels() as f32;
        let descent = html_line.descent().to_pixels() as f32;
        let atoms = html_line
            .text_range()
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
            })
            .collect();
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
        self.html_line.bounds().width().to_pixels() as f32
    }

    pub fn raw_text_length(&self) -> usize {
        self.html_line.text_range().len()
    }

    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
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
    fallback: EditText<'gc>,
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
                fallback,
                movie,
            },
        ))
    }

    pub fn line(self) -> Ref<'gc, TextLineLayout<'gc>> {
        Gc::as_ref(self.0).line.borrow()
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
                fallback: self.0.fallback,
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
        self.0.fallback.render_self(context);
    }

    fn self_bounds(self, mode: BoundsMode) -> Rectangle<Twips> {
        self.0.fallback.self_bounds(mode)
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
