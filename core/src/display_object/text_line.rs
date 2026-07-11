//! The TextLine display object, backing flash.text.engine.TextLine.

use crate::avm1::Object as Avm1Object;
use crate::avm2::StageObject as Avm2StageObject;
use crate::avm2::object::TextBlockObject;
use crate::backend::ui::MouseCursor;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::interactive::{InteractiveObjectBase, TInteractiveObject};
use crate::display_object::{
    Avm2MousePick, BoundsMode, DisplayObjectBase, EditText, InteractiveObject,
};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::Lock;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_common::avm_string::AvmString;
use ruffle_common::utils::HasPrefixField;
use ruffle_macros::istr;
use std::cell::Cell;
use std::sync::Arc;

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

    text_block: Lock<Option<TextBlockObject<'gc>>>,

    specified_width: Cell<f64>,
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
                text_block: Lock::new(None),
                specified_width: Cell::new(0.0),
            },
        ))
    }

    pub fn measure_text(self, context: &mut UpdateContext<'gc>) -> (Twips, Twips) {
        self.0.fallback.measure_text(context)
    }

    pub fn validity(self) -> AvmString<'gc> {
        self.0.validity.get()
    }

    pub fn set_validity(self, validity: AvmString<'gc>, context: &mut UpdateContext<'gc>) {
        unlock!(Gc::write(context.gc(), self.0), TextLineData, validity).set(validity);
    }

    pub fn text_block(self) -> Option<TextBlockObject<'gc>> {
        self.0.text_block.get()
    }

    pub fn set_text_block(self, text_block: Option<TextBlockObject<'gc>>, mc: &Mutation<'gc>) {
        unlock!(Gc::write(mc, self.0), TextLineData, text_block).set(text_block);
    }

    pub fn specified_width(self) -> f64 {
        self.0.specified_width.get()
    }

    pub fn set_specified_width(self, value: f64) {
        self.0.specified_width.set(value);
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
                text_block: Lock::new(self.0.text_block.get()),
                specified_width: Cell::new(self.0.specified_width.get()),
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

    fn post_instantiation(
        self,
        context: &mut UpdateContext<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        _run_frame: bool,
    ) {
        self.set_default_instance_name(context);
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
