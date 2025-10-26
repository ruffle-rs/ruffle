use crate::avm2::Activation;
use crate::avm2::StageObject as Avm2StageObject;
use crate::context::RenderContext;
use crate::context::UpdateContext;
use crate::display_object::InteractiveObject;
use crate::display_object::TInteractiveObject;
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;

use crate::display_object::container::ChildContainer;
use crate::display_object::interactive::InteractiveObjectBase;
use crate::tag_utils::SwfMovie;
use crate::utils::HasPrefixField;
use core::fmt;
use gc_arena::barrier::unlock;
use gc_arena::lock::{Lock, RefLock};
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::cell::{Ref, RefMut};
use std::sync::Arc;

use super::interactive::Avm2MousePick;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct LoaderDisplay<'gc>(Gc<'gc, LoaderDisplayData<'gc>>);

impl fmt::Debug for LoaderDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoaderDisplay")
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct LoaderDisplayData<'gc> {
    base: InteractiveObjectBase<'gc>,
    container: RefLock<ChildContainer<'gc>>,
    avm2_object: Lock<Option<Avm2StageObject<'gc>>>,
    movie: Arc<SwfMovie>,
}

impl<'gc> LoaderDisplay<'gc> {
    pub fn empty(activation: &mut Activation<'_, 'gc>, movie: Arc<SwfMovie>) -> Self {
        let obj = LoaderDisplay(Gc::new(
            activation.gc(),
            LoaderDisplayData {
                base: Default::default(),
                container: RefLock::new(ChildContainer::new(&movie)),
                avm2_object: Lock::new(None),
                movie,
            },
        ));

        obj.set_placed_by_avm2_script(true);
        activation.context.orphan_manager.add_orphan_obj(obj.into());
        obj
    }

    pub fn downgrade(self) -> LoaderDisplayWeak<'gc> {
        LoaderDisplayWeak(Gc::downgrade(self.0))
    }
}

impl<'gc> TDisplayObject<'gc> for LoaderDisplay<'gc> {
    fn base(self) -> Gc<'gc, DisplayObjectBase<'gc>> {
        HasPrefixField::as_prefix_gc(self.raw_interactive())
    }

    fn instantiate(self, gc_context: &Mutation<'gc>) -> DisplayObject<'gc> {
        Self(Gc::new(gc_context, self.0.as_ref().clone())).into()
    }

    fn id(self) -> CharacterId {
        u16::MAX
    }

    fn render_self(self, context: &mut RenderContext<'_, 'gc>) {
        self.render_children(context);
    }

    fn self_bounds(self) -> Rectangle<Twips> {
        Default::default()
    }

    fn object1(self) -> Option<crate::avm1::Object<'gc>> {
        None
    }

    fn object2(self) -> Option<Avm2StageObject<'gc>> {
        self.0.avm2_object.get()
    }

    fn set_object2(self, context: &mut UpdateContext<'gc>, to: Avm2StageObject<'gc>) {
        let mc = context.gc();
        unlock!(Gc::write(mc, self.0), LoaderDisplayData, avm2_object).set(Some(to))
    }

    fn enter_frame(self, context: &mut UpdateContext<'gc>) {
        let skip_frame = self.base().should_skip_next_enter_frame();
        for child in self.iter_render_list() {
            // See MovieClip::enter_frame for an explanation of this.
            if skip_frame {
                child.base().set_skip_next_enter_frame(true);
            }
            child.enter_frame(context);
        }
        self.base().set_skip_next_enter_frame(false);
    }

    fn construct_frame(self, context: &mut UpdateContext<'gc>) {
        for child in self.iter_render_list() {
            child.construct_frame(context);
        }
    }

    fn movie(self) -> Arc<SwfMovie> {
        self.0.movie.clone()
    }

    fn on_parent_removed(self, context: &mut UpdateContext<'gc>) {
        if self.movie().is_action_script_3() {
            context.orphan_manager.add_orphan_obj(self.into())
        }
    }
}

impl<'gc> TInteractiveObject<'gc> for LoaderDisplay<'gc> {
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
        if !self.visible() {
            return ClipEventResult::NotHandled;
        }

        ClipEventResult::Handled
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
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        // Don't do anything if run in an AVM2 context.
        if self.as_displayobject().movie().is_action_script_3() {
            return None;
        }

        for child in self.iter_render_list().rev() {
            if let Some(int) = child.as_interactive() {
                if int.as_displayobject().movie().is_action_script_3() {
                    let avm2_result = int.mouse_pick_avm2(context, point, require_button_mode);
                    if let Avm2MousePick::Hit(result) = avm2_result {
                        return Some(result);
                    }
                } else if let Some(result) =
                    int.mouse_pick_avm1(context, point, require_button_mode)
                {
                    return Some(result);
                }
            }
        }

        None
    }

    fn mouse_pick_avm2(
        self,
        context: &mut UpdateContext<'gc>,
        point: Point<Twips>,
        require_button_mode: bool,
    ) -> Avm2MousePick<'gc> {
        // Don't do anything if run in an AVM1 context.
        if !self.as_displayobject().movie().is_action_script_3() {
            return Avm2MousePick::Miss;
        }

        let mut options = HitTestOptions::SKIP_INVISIBLE;
        options.set(HitTestOptions::SKIP_MASK, self.maskee().is_none());

        // We have at most one child
        if let Some(child) = self.iter_render_list().next() {
            if let Some(int) = child.as_interactive() {
                if int.as_displayobject().movie().is_action_script_3() {
                    return int
                        .mouse_pick_avm2(context, point, require_button_mode)
                        .combine_with_parent(self.into());
                } else {
                    let avm1_result = int.mouse_pick_avm1(context, point, require_button_mode);
                    if let Some(result) = avm1_result {
                        return Avm2MousePick::Hit(result);
                    } else {
                        return Avm2MousePick::Miss;
                    }
                }
            } else if child.hit_test_shape(context, point, options) {
                if self.mouse_enabled() {
                    return Avm2MousePick::Hit(self.into());
                } else {
                    return Avm2MousePick::PropagateToParent;
                }
            }
        }
        Avm2MousePick::Miss
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for LoaderDisplay<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        self.0.container.borrow()
    }

    fn raw_container_mut(&self, gc_context: &Mutation<'gc>) -> RefMut<'_, ChildContainer<'gc>> {
        unlock!(Gc::write(gc_context, self.0), LoaderDisplayData, container).borrow_mut()
    }
}

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct LoaderDisplayWeak<'gc>(GcWeak<'gc, LoaderDisplayData<'gc>>);

impl<'gc> LoaderDisplayWeak<'gc> {
    pub fn upgrade(self, mc: &Mutation<'gc>) -> Option<LoaderDisplay<'gc>> {
        self.0.upgrade(mc).map(LoaderDisplay)
    }

    pub fn as_ptr(self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }
}
