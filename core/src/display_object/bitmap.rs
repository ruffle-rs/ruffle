//! Bitmap display object

use crate::avm1;
use crate::avm2::{
    Activation as Avm2Activation, ClassObject as Avm2ClassObject, Object as Avm2Object,
    StageObject as Avm2StageObject, Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::{DisplayObjectBase, DisplayObjectPtr, TDisplayObject};
use crate::prelude::*;
use crate::vminterface::Instantiator;
use gc_arena::{Collect, Gc, GcCell, MutationContext};
use ruffle_render::bitmap::BitmapHandle;
use std::cell::{Ref, RefMut};

/// A Bitmap display object is a raw bitamp on the stage.
/// This can only be instanitated on the display list in SWFv9 AVM2 files.
/// In AVM1, this is only a library symbol that is referenced by `Graphic`.
/// Normally bitmaps are drawn in Flash as part of a Shape tag (`Graphic`),
/// but starting in AVM2, a raw `Bitmap` display object can be created
/// with the `PlaceObject3` tag.
/// It can also be created in ActionScript using the `Bitmap` class.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Bitmap<'gc>(GcCell<'gc, BitmapData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct BitmapData<'gc> {
    base: DisplayObjectBase<'gc>,
    static_data: Gc<'gc, BitmapStatic>,

    /// The current bitmap data object.
    bitmap_data: Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>>,

    /// The current bitmap handle.
    ///
    /// This needs to be cached separately from the associated bitmap data so
    /// that it can be accessed without a mutation context.
    ///
    /// If this is `None`, then the bitmap does not render anything.
    bitmap_handle: Option<BitmapHandle>,

    /// Whether or not bitmap smoothing is enabled.
    smoothing: bool,

    /// The AVM2 side of this object.
    ///
    /// AVM1 code cannot directly reference `Bitmap`s, so this does not support
    /// storing an AVM1 object.
    avm2_object: Option<Avm2Object<'gc>>,

    /// The AVM2 class for the BitmapData associated with this object.
    ///
    /// When bitmaps are instantiated by the timeline, they are constructed as
    /// AVM2's `Bitmap` class, and then they are associated with `BitmapData`
    /// that is constructed from the given symbol class.
    ///
    /// This association is unusual relative to other things that use AS3
    /// linkage, where the symbol class usually directly represents the symbol.
    avm2_bitmapdata_class: Option<Avm2ClassObject<'gc>>,
}

impl<'gc> Bitmap<'gc> {
    /// Create a `Bitmap` with dynamic bitmap data.
    ///
    /// If `bitmap_data` is provided, the associated `bitmap_handle` must match
    /// the same handle that the data has provided. If it does not match, then
    /// this `Bitmap` will render the wrong data when added to the display
    /// list. If no data is provided then you are free to add whatever handle
    /// you like.
    pub fn new_with_bitmap_data(
        context: &mut UpdateContext<'_, 'gc, '_>,
        id: CharacterId,
        bitmap_handle: Option<BitmapHandle>,
        width: u16,
        height: u16,
        bitmap_data: Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>>,
        smoothing: bool,
    ) -> Self {
        //NOTE: We do *not* solicit a handle from the `bitmap_data` at this
        //time due to mutable borrowing issues.

        Bitmap(GcCell::allocate(
            context.gc_context,
            BitmapData {
                base: Default::default(),
                static_data: Gc::allocate(context.gc_context, BitmapStatic { id, width, height }),
                bitmap_data,
                bitmap_handle,
                smoothing,
                avm2_object: None,
                avm2_bitmapdata_class: None,
            },
        ))
    }

    /// Create a `Bitmap` with static bitmap data only.
    pub fn new(
        context: &mut UpdateContext<'_, 'gc, '_>,
        id: CharacterId,
        bitmap_handle: BitmapHandle,
        width: u16,
        height: u16,
    ) -> Self {
        Self::new_with_bitmap_data(context, id, Some(bitmap_handle), width, height, None, true)
    }

    #[allow(dead_code)]
    pub fn bitmap_handle(self) -> Option<BitmapHandle> {
        self.0.read().bitmap_handle
    }

    pub fn width(self) -> u16 {
        let read = self.0.read();

        read.bitmap_data
            .map(|bd| bd.read().width() as u16)
            .unwrap_or_else(|| read.static_data.width)
    }

    pub fn height(self) -> u16 {
        let read = self.0.read();

        read.bitmap_data
            .map(|bd| bd.read().height() as u16)
            .unwrap_or_else(|| read.static_data.height)
    }

    /// Retrieve the bitmap data associated with this `Bitmap`.
    pub fn bitmap_data(self) -> Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>> {
        self.0.read().bitmap_data
    }

    /// Associate this `Bitmap` with new `BitmapData`.
    ///
    /// Once associated with the new data, the reported width, height, and
    /// bitmap handle of this display object will change to match the given
    /// bitmap data.
    ///
    /// This also forces the `BitmapData` to be sent to the rendering backend,
    /// if that has not already been done.
    pub fn set_bitmap_data(
        self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        bitmap_data: Option<GcCell<'gc, crate::bitmap::bitmap_data::BitmapData<'gc>>>,
    ) {
        if let Some(bitmap_data) = bitmap_data {
            let bitmap_handle = bitmap_data
                .write(context.gc_context)
                .bitmap_handle(context.renderer);

            let mut write = self.0.write(context.gc_context);

            write.bitmap_data = Some(bitmap_data);
            if let Some(bitmap_handle) = bitmap_handle {
                write.bitmap_handle = Some(bitmap_handle);
            }
        } else {
            let mut write = self.0.write(context.gc_context);

            write.bitmap_data = None;
            write.bitmap_handle = None;
        }
    }

    pub fn avm2_bitmapdata_class(self) -> Option<Avm2ClassObject<'gc>> {
        self.0.read().avm2_bitmapdata_class
    }

    pub fn set_avm2_bitmapdata_class(
        self,
        mc: MutationContext<'gc, '_>,
        class: Avm2ClassObject<'gc>,
    ) {
        self.0.write(mc).avm2_bitmapdata_class = Some(class);
    }

    pub fn smoothing(self) -> bool {
        self.0.read().smoothing
    }

    pub fn set_smoothing(self, mc: MutationContext<'gc, '_>, smoothing: bool) {
        self.0.write(mc).smoothing = smoothing;
    }
}

impl<'gc> TDisplayObject<'gc> for Bitmap<'gc> {
    fn base(&self) -> Ref<DisplayObjectBase<'gc>> {
        Ref::map(self.0.read(), |r| &r.base)
    }

    fn base_mut<'a>(&'a self, mc: MutationContext<'gc, '_>) -> RefMut<'a, DisplayObjectBase<'gc>> {
        RefMut::map(self.0.write(mc), |w| &mut w.base)
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc> {
        Self(GcCell::allocate(gc_context, self.0.read().clone())).into()
    }

    fn as_ptr(&self) -> *const DisplayObjectPtr {
        self.0.as_ptr() as *const DisplayObjectPtr
    }

    fn id(&self) -> CharacterId {
        self.0.read().static_data.id
    }

    fn self_bounds(&self) -> BoundingBox {
        BoundingBox {
            x_min: Twips::ZERO,
            y_min: Twips::ZERO,
            x_max: Twips::from_pixels(Bitmap::width(*self).into()),
            y_max: Twips::from_pixels(Bitmap::height(*self).into()),
            valid: true,
        }
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _init_object: Option<avm1::Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if context.is_action_script_3() {
            let mut activation = Avm2Activation::from_nothing(context.reborrow());
            let bitmap = activation.avm2().classes().bitmap;
            match Avm2StageObject::for_display_object_childless(
                &mut activation,
                (*self).into(),
                bitmap,
            ) {
                Ok(object) => {
                    self.0.write(activation.context.gc_context).avm2_object = Some(object.into())
                }
                Err(e) => log::error!("Got error when creating AVM2 side of bitmap: {}", e),
            }
        } else {
            context
                .avm1
                .add_to_exec_list(context.gc_context, (*self).into());
        }

        if run_frame {
            self.run_frame(context);
        }
    }

    fn run_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if let (Some(bitmap_data), Some(bitmap_handle)) =
            (&self.0.read().bitmap_data, self.0.read().bitmap_handle)
        {
            let bd = bitmap_data.read();
            if bd.dirty() {
                let _ = context.renderer.update_texture(
                    bitmap_handle,
                    bd.width(),
                    bd.height(),
                    bd.pixels_rgba(),
                );
                drop(bd);
                bitmap_data.write(context.gc_context).set_dirty(false);
            }
        }
    }

    fn render_self(&self, context: &mut RenderContext) {
        if !self.world_bounds().intersects(&context.stage.view_bounds()) {
            // Off-screen; culled
            return;
        }

        let bitmap_data = self.0.read();
        if let Some(bitmap_handle) = bitmap_data.bitmap_handle {
            context.renderer.render_bitmap(
                bitmap_handle,
                context.transform_stack.transform(),
                bitmap_data.smoothing,
            );
        }
    }

    fn object2(&self) -> Avm2Value<'gc> {
        self.0
            .read()
            .avm2_object
            .map(|o| o.into())
            .unwrap_or(Avm2Value::Undefined)
    }

    fn as_bitmap(self) -> Option<Bitmap<'gc>> {
        Some(self)
    }
}

/// Static data shared between all instances of a bitmap.
#[derive(Clone, Collect)]
#[collect(no_drop)]
struct BitmapStatic {
    id: CharacterId,
    width: u16,
    height: u16,
}
