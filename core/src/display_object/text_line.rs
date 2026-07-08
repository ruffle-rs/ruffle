//! The TextLine display object, backing flash.text.engine.TextLine.

use crate::avm2::StageObject as Avm2StageObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{
    Avm2MousePick, BoundsMode, DisplayObjectBase, EditText, InteractiveObject,
};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::avm_string::AvmString;
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use ruffle_render::transform::Transform;
use std::cell::Cell;
use std::sync::Arc;

/// Metrics of a laid-out text line, in the line's own coordinate space.
///
/// The origin of a `TextLine` is the start of its baseline: the text extends
/// `ascent` above and `descent` below y=0.
#[derive(Clone, Copy, Collect, Default)]
#[collect(require_static)]
pub struct LineMetrics {
    pub ascent: Twips,
    pub descent: Twips,
    pub text_width: Twips,
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
    fallback: EditText<'gc>,
    #[collect(require_static)]
    movie: Arc<SwfMovie>,

    /// Validity can be any user-defined string, we can't use an enum here.
    ///
    /// See [`TextLineValidity`] for the known values of validity.
    validity: Lock<AvmString<'gc>>,

    /// Metrics of this line, calculated when the line is (re)created.
    metrics: Cell<LineMetrics>,
}

impl<'gc> TextLine<'gc> {
    pub fn new(
        context: &mut UpdateContext<'gc>,
        movie: Arc<SwfMovie>,
        fallback: EditText<'gc>,
    ) -> Self {
        TextLine(Gc::new(
            context.gc(),
            TextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                fallback,
                movie,
                validity: Lock::new(istr!(context, "valid")),
                metrics: Cell::new(LineMetrics::default()),
            },
        ))
    }

    pub fn measure_text(self, context: &mut UpdateContext<'gc>) -> (Twips, Twips) {
        self.0.fallback.measure_text(context)
    }

    /// The `EditText` this line delegates layout and rendering to.
    pub fn fallback(self) -> EditText<'gc> {
        self.0.fallback
    }

    pub fn metrics(self) -> LineMetrics {
        self.0.metrics.get()
    }

    pub fn set_metrics(self, metrics: LineMetrics) {
        self.0.metrics.set(metrics);
    }

    /// Offset translating the fallback `EditText` (whose origin is its
    /// top-left corner, inset by the gutter) so that this line's origin
    /// is the start of the text baseline, like in Flash Player.
    fn fallback_offset(self) -> (Twips, Twips) {
        let gutter = EditText::GUTTER;
        let ascent = self.0.metrics.get().ascent;
        (-gutter, -(gutter + ascent))
    }

    pub fn validity(self) -> AvmString<'gc> {
        self.0.validity.get()
    }

    pub fn set_validity(self, validity: AvmString<'gc>, context: &mut UpdateContext<'gc>) {
        unlock!(Gc::write(context.gc(), self.0), TextLineData, validity).set(validity);
    }
}

impl<'gc> TDisplayObject<'gc> for TextLine<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        let interactive: Gc<'gc, InteractiveObjectBase<'gc>> = HasPrefixField::as_prefix_gc(self.0);
        HasPrefixField::as_prefix_gc(interactive)
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(
            gc_context,
            TextLineData {
                base: Default::default(),
                avm2_object: Lock::new(None),
                fallback: self.0.fallback,
                movie: self.0.movie.clone(),
                validity: Lock::new(self.0.validity.get()),
                metrics: Cell::new(self.0.metrics.get()),
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
        let (dx, dy) = self.fallback_offset();
        context.transform_stack.push(&Transform {
            matrix: Matrix::translate(dx, dy),
            ..Default::default()
        });
        self.0.fallback.render_self(context);
        context.transform_stack.pop();
    }

    fn self_bounds(self, mode: BoundsMode) -> Rectangle<Twips> {
        let (dx, dy) = self.fallback_offset();
        let bounds = self.0.fallback.self_bounds(mode);
        Rectangle {
            x_min: bounds.x_min + dx,
            x_max: bounds.x_max + dx,
            y_min: bounds.y_min + dy,
            y_max: bounds.y_max + dy,
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
