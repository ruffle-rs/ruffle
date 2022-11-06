use crate::avm2::Object as Avm2Object;
use crate::context::RenderContext;
use crate::context::UpdateContext;
use crate::display_object::InteractiveObject;
use crate::display_object::TInteractiveObject;
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::events::{ClipEvent, ClipEventResult};
use crate::prelude::*;

use crate::display_object::container::ChildContainer;
use crate::display_object::interactive::InteractiveObjectBase;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};

#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct LoaderDisplay<'gc>(GcCell<'gc, LoaderDisplayData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct LoaderDisplayData<'gc> {
    base: InteractiveObjectBase<'gc>,
    container: ChildContainer<'gc>,
    avm2_object: Avm2Object<'gc>,
}

impl<'gc> LoaderDisplay<'gc> {
    pub fn new_with_avm2(
        gc_context: MutationContext<'gc, '_>,
        avm2_object: Avm2Object<'gc>,
    ) -> Self {
        LoaderDisplay(GcCell::allocate(
            gc_context,
            LoaderDisplayData {
                base: Default::default(),
                container: ChildContainer::new(),
                avm2_object,
            },
        ))
    }
}

impl<'gc> TDisplayObject<'gc> for LoaderDisplay<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        u16::MAX
    }

    fn render_self(&self, context: &mut RenderContext<'_, 'gc, '_>) {
        self.render_children(context);
    }

    fn self_bounds(&self) -> BoundingBox {
        Default::default()
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0.read().avm2_object.into()
    }

    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        Some(self.into())
    }

    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        Some(self.into())
    }
}

impl<'gc> TInteractiveObject<'gc> for LoaderDisplay<'gc> {
    fn ibase(&self) -> Ref<InteractiveObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn ibase_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<InteractiveObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn as_displayobject(self) -> DisplayObject<'gc> {
        self.into()
    }

    fn filter_clip_event(self, _event: ClipEvent) -> ClipEventResult {
        ClipEventResult::NotHandled
    }
    fn event_dispatch(
        self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _event: ClipEvent<'gc>,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        pos: (Twips, Twips),
        require_button_mode: bool,
    ) -> Option<InteractiveObject<'gc>> {
        for child in self.iter_render_list().rev() {
            if let Some(int) = child.as_interactive() {
                if let Some(result) = int.mouse_pick(context, pos, require_button_mode) {
                    return Some(result);
                }
            }
        }

        None
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for LoaderDisplay<'gc> {
    fn raw_container(&self) -> Ref<'_, ChildContainer<'gc>> {
        Ref::map(self.0.read(), |this| &this.container)
    }

    fn raw_container_mut(
        &self,
        gc_context: MutationContext<'gc, '_>,
    ) -> RefMut<'_, ChildContainer<'gc>> {
        RefMut::map(self.0.write(gc_context), |this| &mut this.container)
    }
}
