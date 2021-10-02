use crate::avm1::{
    Error as Avm1Error, Object as Avm1Object, TObject as Avm1TObject, Value as Avm1Value,
};
use crate::avm2::{
    Activation as Avm2Activation, Avm2, Error as Avm2Error, Event as Avm2Event,
    Namespace as Avm2Namespace, Object as Avm2Object, QName as Avm2QName, TObject as Avm2TObject,
    Value as Avm2Value,
};
use crate::context::{RenderContext, UpdateContext};
use crate::drawing::Drawing;
use crate::player::NEWEST_PLAYER_VERSION;
use crate::prelude::*;
use crate::string::AvmString;
use crate::tag_utils::SwfMovie;
use crate::transform::Transform;
use crate::types::{Degrees, Percent};
use crate::vminterface::{AvmType, Instantiator};
use bitflags::bitflags;
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;
use std::sync::Arc;
use swf::Fixed8;

mod avm1_button;
mod avm2_button;
mod bitmap;
mod container;
mod edit_text;
mod graphic;
mod interactive;
mod morph_shape;
mod movie_clip;
mod stage;
mod text;
mod video;

use crate::avm1::activation::Activation;
use crate::backend::ui::MouseCursor;
pub use crate::display_object::container::{
    DisplayObjectContainer, Lists, TDisplayObjectContainer,
};
use crate::events::{ClipEvent, ClipEventResult};
pub use avm1_button::{Avm1Button, ButtonState, ButtonTracking};
pub use avm2_button::Avm2Button;
pub use bitmap::Bitmap;
pub use edit_text::{AutoSizeMode, EditText, TextSelection};
pub use graphic::Graphic;
pub use interactive::{InteractiveObject, TInteractiveObject};
pub use morph_shape::{MorphShape, MorphShapeStatic};
pub use movie_clip::{MovieClip, Scene};
pub use stage::{Stage, StageAlign, StageQuality, StageScaleMode};
pub use text::Text;
pub use video::Video;

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct DisplayObjectBase<'gc> {
    parent: Option<DisplayObject<'gc>>,
    place_frame: u16,
    depth: Depth,
    transform: Transform,
    name: AvmString<'gc>,
    clip_depth: Depth,

    // Cached transform properties `_xscale`, `_yscale`, `_rotation`.
    // These are expensive to calculate, so they will be calculated and cached
    // when AS requests one of these properties.
    //
    // `_xscale` and `_yscale` are stored in units of percentages to avoid
    // floating-point precision errors with movies that accumulate onto the
    // scale parameters.
    rotation: Degrees,
    scale_x: Percent,
    scale_y: Percent,
    skew: f64,

    /// The previous display object in order of AVM1 execution.
    ///
    /// `None` in an AVM2 movie.
    prev_avm1_clip: Option<DisplayObject<'gc>>,

    /// The next display object in order of execution.
    ///
    /// `None` in an AVM2 movie.
    next_avm1_clip: Option<DisplayObject<'gc>>,

    /// The sound transform of sounds playing via this display object.
    sound_transform: SoundTransform,

    /// The display object that we are being masked by.
    masker: Option<DisplayObject<'gc>>,

    /// The display object we are currently masking.
    maskee: Option<DisplayObject<'gc>>,

    /// Bit flags for various display object properties.
    flags: DisplayObjectFlags,
}

impl<'gc> Default for DisplayObjectBase<'gc> {
    fn default() -> Self {
        Self {
            parent: Default::default(),
            place_frame: Default::default(),
            depth: Default::default(),
            transform: Default::default(),
            name: Default::default(),
            clip_depth: Default::default(),
            rotation: Degrees::from_radians(0.0),
            scale_x: Percent::from_unit(1.0),
            scale_y: Percent::from_unit(1.0),
            skew: 0.0,
            prev_avm1_clip: None,
            next_avm1_clip: None,
            masker: None,
            maskee: None,
            sound_transform: Default::default(),
            flags: DisplayObjectFlags::VISIBLE,
        }
    }
}

#[allow(dead_code)]
impl<'gc> DisplayObjectBase<'gc> {
    /// Reset all properties that would be adjusted by a movie load.
    fn reset_for_movie_load(&mut self) {
        let flags_to_keep = self.flags & DisplayObjectFlags::LOCK_ROOT;
        self.flags = flags_to_keep | DisplayObjectFlags::VISIBLE;
    }

    fn id(&self) -> CharacterId {
        0
    }

    fn depth(&self) -> Depth {
        self.depth
    }

    fn set_depth(&mut self, depth: Depth) {
        self.depth = depth;
    }

    fn place_frame(&self) -> u16 {
        self.place_frame
    }

    fn set_place_frame(&mut self, frame: u16) {
        self.place_frame = frame;
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn matrix(&self) -> &Matrix {
        &self.transform.matrix
    }

    fn matrix_mut(&mut self) -> &mut Matrix {
        &mut self.transform.matrix
    }

    fn set_matrix(&mut self, matrix: &Matrix) {
        self.transform.matrix = *matrix;
        self.flags -= DisplayObjectFlags::SCALE_ROTATION_CACHED;
    }

    fn color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }

    fn color_transform_mut(&mut self) -> &mut ColorTransform {
        &mut self.transform.color_transform
    }

    fn set_color_transform(&mut self, color_transform: &ColorTransform) {
        self.transform.color_transform = *color_transform;
    }

    fn x(&self) -> f64 {
        self.transform.matrix.tx.to_pixels()
    }

    fn set_x(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.transform.matrix.tx = Twips::from_pixels(value)
    }

    fn y(&self) -> f64 {
        self.transform.matrix.ty.to_pixels()
    }

    fn set_y(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.transform.matrix.ty = Twips::from_pixels(value)
    }

    /// Caches the scale and rotation factors for this display object, if necessary.
    /// Calculating these requires heavy trig ops, so we only do it when `_xscale`, `_yscale` or
    /// `_rotation` is accessed.
    fn cache_scale_rotation(&mut self) {
        if !self
            .flags
            .contains(DisplayObjectFlags::SCALE_ROTATION_CACHED)
        {
            let (a, b, c, d) = (
                f64::from(self.transform.matrix.a),
                f64::from(self.transform.matrix.b),
                f64::from(self.transform.matrix.c),
                f64::from(self.transform.matrix.d),
            );
            // If this object's transform matrix is:
            // [[a c tx]
            //  [b d ty]]
            // After transformation, the X-axis and Y-axis will turn into the column vectors x' = <a, b> and y' = <c, d>.
            // We derive the scale, rotation, and skew values from these transformed axes.
            // The skew value is not exposed by ActionScript, but is remembered internally.
            // xscale = len(x')
            // yscale = len(y')
            // rotation = atan2(b, a)  (the rotation of x' from the normal x-axis).
            // skew = atan2(-c, d) - atan2(b, a)  (the signed difference between y' and x' rotation)

            // This can produce some surprising results due to the overlap between flipping/rotation/skewing.
            // For example, in Flash, using Modify->Transform->Flip Horizontal and then tracing _xscale, _yscale, and _rotation
            // will output 100, 100, and 180. (a horizontal flip could also be a 180 degree skew followed by 180 degree rotation!)
            let rotation_x = f64::atan2(b, a);
            let rotation_y = f64::atan2(-c, d);
            let scale_x = f64::sqrt(a * a + b * b);
            let scale_y = f64::sqrt(c * c + d * d);
            self.rotation = Degrees::from_radians(rotation_x);
            self.scale_x = Percent::from_unit(scale_x);
            self.scale_y = Percent::from_unit(scale_y);
            self.skew = rotation_y - rotation_x;
            self.flags |= DisplayObjectFlags::SCALE_ROTATION_CACHED;
        }
    }

    fn set_scale(&mut self, scale_x: f32, scale_y: f32, rotation: f32) {
        self.cache_scale_rotation();
        let mut matrix = &mut self.transform.matrix;
        let rotation = rotation.to_radians();
        let cos_x = f32::cos(rotation);
        let sin_x = f32::sin(rotation);
        self.scale_x = Percent::from_unit(scale_x.into());
        self.scale_y = Percent::from_unit(scale_y.into());
        self.rotation = Degrees::from_radians(rotation.into());
        matrix.a = (scale_x * cos_x) as f32;
        matrix.b = (scale_x * sin_x) as f32;
        matrix.c = (scale_y * -sin_x) as f32;
        matrix.d = (scale_y * cos_x) as f32;
    }

    fn rotation(&mut self) -> Degrees {
        self.cache_scale_rotation();
        self.rotation
    }

    fn set_rotation(&mut self, degrees: Degrees) {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.rotation = degrees;
        let cos_x = f64::cos(degrees.into_radians());
        let sin_x = f64::sin(degrees.into_radians());
        let cos_y = f64::cos(degrees.into_radians() + self.skew);
        let sin_y = f64::sin(degrees.into_radians() + self.skew);
        let mut matrix = &mut self.transform.matrix;
        matrix.a = (self.scale_x.into_unit() * cos_x) as f32;
        matrix.b = (self.scale_x.into_unit() * sin_x) as f32;
        matrix.c = (self.scale_y.into_unit() * -sin_y) as f32;
        matrix.d = (self.scale_y.into_unit() * cos_y) as f32;
    }

    fn scale_x(&mut self) -> Percent {
        self.cache_scale_rotation();
        self.scale_x
    }

    fn set_scale_x(&mut self, value: Percent) {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_x = value;
        let cos = f64::cos(self.rotation.into_radians());
        let sin = f64::sin(self.rotation.into_radians());
        let mut matrix = &mut self.transform.matrix;
        matrix.a = (cos * value.into_unit()) as f32;
        matrix.b = (sin * value.into_unit()) as f32;
    }

    fn scale_y(&mut self) -> Percent {
        self.cache_scale_rotation();
        self.scale_y
    }

    fn set_scale_y(&mut self, value: Percent) {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_y = value;
        let cos = f64::cos(self.rotation.into_radians() + self.skew);
        let sin = f64::sin(self.rotation.into_radians() + self.skew);
        let mut matrix = &mut self.transform.matrix;
        matrix.c = (-sin * value.into_unit()) as f32;
        matrix.d = (cos * value.into_unit()) as f32;
    }

    fn name(&self) -> AvmString<'gc> {
        self.name
    }

    fn set_name(&mut self, name: AvmString<'gc>) {
        self.name = name;
    }

    fn alpha(&self) -> f64 {
        f64::from(self.color_transform().a_mult)
    }

    fn set_alpha(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.color_transform_mut().a_mult = Fixed8::from_f64(value)
    }

    fn clip_depth(&self) -> Depth {
        self.clip_depth
    }

    fn set_clip_depth(&mut self, depth: Depth) {
        self.clip_depth = depth;
    }

    fn parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent
    }

    fn set_parent(&mut self, parent: Option<DisplayObject<'gc>>) {
        self.parent = parent;
    }

    fn prev_avm1_clip(&self) -> Option<DisplayObject<'gc>> {
        self.prev_avm1_clip
    }

    fn set_prev_avm1_clip(&mut self, node: Option<DisplayObject<'gc>>) {
        self.prev_avm1_clip = node;
    }

    fn next_avm1_clip(&self) -> Option<DisplayObject<'gc>> {
        self.next_avm1_clip
    }

    fn set_next_avm1_clip(&mut self, node: Option<DisplayObject<'gc>>) {
        self.next_avm1_clip = node;
    }

    fn removed(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::REMOVED)
    }

    fn set_removed(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::REMOVED, value);
    }

    fn sound_transform(&self) -> &SoundTransform {
        &self.sound_transform
    }

    fn set_sound_transform(&mut self, sound_transform: SoundTransform) {
        self.sound_transform = sound_transform;
    }

    fn visible(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::VISIBLE)
    }

    fn set_visible(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::VISIBLE, value);
    }

    fn is_root(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::IS_ROOT)
    }

    fn set_is_root(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::IS_ROOT, value);
    }

    fn lock_root(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::LOCK_ROOT)
    }

    fn set_lock_root(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::LOCK_ROOT, value);
    }

    fn transformed_by_script(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::TRANSFORMED_BY_SCRIPT)
    }

    fn set_transformed_by_script(&mut self, value: bool) {
        self.flags
            .set(DisplayObjectFlags::TRANSFORMED_BY_SCRIPT, value);
    }

    fn placed_by_script(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::PLACED_BY_SCRIPT)
    }

    fn set_placed_by_script(&mut self, value: bool) {
        self.flags.set(DisplayObjectFlags::PLACED_BY_SCRIPT, value);
    }

    fn instantiated_by_timeline(&self) -> bool {
        self.flags
            .contains(DisplayObjectFlags::INSTANTIATED_BY_TIMELINE)
    }

    fn set_instantiated_by_timeline(&mut self, value: bool) {
        self.flags
            .set(DisplayObjectFlags::INSTANTIATED_BY_TIMELINE, value);
    }

    fn swf_version(&self) -> u8 {
        self.parent
            .map(|p| p.swf_version())
            .unwrap_or(NEWEST_PLAYER_VERSION)
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        self.parent.and_then(|p| p.movie())
    }

    fn masker(&self) -> Option<DisplayObject<'gc>> {
        self.masker
    }
    fn set_masker(&mut self, node: Option<DisplayObject<'gc>>) {
        self.masker = node;
    }

    fn maskee(&self) -> Option<DisplayObject<'gc>> {
        self.maskee
    }
    fn set_maskee(&mut self, node: Option<DisplayObject<'gc>>) {
        self.maskee = node;
    }
}

pub fn render_base<'gc>(this: DisplayObject<'gc>, context: &mut RenderContext<'_, 'gc>) {
    if this.maskee().is_some() {
        return;
    }
    context.transform_stack.push(&*this.transform());

    let mask = this.masker();
    let mut mask_transform = crate::transform::Transform::default();
    if let Some(m) = mask {
        mask_transform.matrix = this.global_to_local_matrix();
        mask_transform.matrix *= m.local_to_global_matrix();
        context.renderer.push_mask();
        context.allow_mask = false;
        context.transform_stack.push(&mask_transform);
        m.render_self(context);
        context.transform_stack.pop();
        context.allow_mask = true;
        context.renderer.activate_mask();
    }
    this.render_self(context);
    if let Some(m) = mask {
        context.renderer.deactivate_mask();
        context.allow_mask = false;
        context.transform_stack.push(&mask_transform);
        m.render_self(context);
        context.transform_stack.pop();
        context.allow_mask = true;
        context.renderer.pop_mask();
    }

    context.transform_stack.pop();
}

#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum DisplayObject<'gc> {
        Stage(Stage<'gc>),
        Bitmap(Bitmap<'gc>),
        Avm1Button(Avm1Button<'gc>),
        Avm2Button(Avm2Button<'gc>),
        EditText(EditText<'gc>),
        Graphic(Graphic<'gc>),
        MorphShape(MorphShape<'gc>),
        MovieClip(MovieClip<'gc>),
        Text(Text<'gc>),
        Video(Video<'gc>),
    }
)]
pub trait TDisplayObject<'gc>:
    'gc + Clone + Copy + Collect + Debug + Into<DisplayObject<'gc>>
{
    fn id(&self) -> CharacterId;
    fn depth(&self) -> Depth;
    fn set_depth(&self, gc_context: MutationContext<'gc, '_>, depth: Depth);

    /// The untransformed inherent bounding box of this object.
    /// These bounds do **not** include child DisplayObjects.
    /// To get the bounds including children, use `bounds`, `local_bounds`, or `world_bounds`.
    ///
    /// Implementors must override this method.
    /// Leaf DisplayObjects should return their bounds.
    /// Composite DisplayObjects that only contain children should return `&Default::default()`
    fn self_bounds(&self) -> BoundingBox;

    /// The untransformed bounding box of this object including children.
    fn bounds(&self) -> BoundingBox {
        self.bounds_with_transform(&Matrix::default())
    }

    /// The local bounding box of this object including children, in its parent's coordinate system.
    fn local_bounds(&self) -> BoundingBox {
        self.bounds_with_transform(&self.matrix())
    }

    /// The world bounding box of this object including children, relative to the stage.
    fn world_bounds(&self) -> BoundingBox {
        self.bounds_with_transform(&self.local_to_global_matrix())
    }

    /// Gets the bounds of this object and all children, transformed by a given matrix.
    /// This function recurses down and transforms the AABB each child before adding
    /// it to the bounding box. This gives a tighter AABB then if we simply transformed
    /// the overall AABB.
    fn bounds_with_transform(&self, matrix: &Matrix) -> BoundingBox {
        let mut bounds = self.self_bounds().transform(matrix);

        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                let matrix = *matrix * *child.matrix();
                bounds.union(&child.bounds_with_transform(&matrix));
            }
        }

        bounds
    }

    fn place_frame(&self) -> u16;
    fn set_place_frame(&self, gc_context: MutationContext<'gc, '_>, frame: u16);

    fn transform(&self) -> Ref<Transform>;
    fn matrix(&self) -> Ref<Matrix>;
    fn matrix_mut(&self, gc_context: MutationContext<'gc, '_>) -> RefMut<Matrix>;
    fn set_matrix(&self, gc_context: MutationContext<'gc, '_>, matrix: &Matrix);
    fn color_transform(&self) -> Ref<ColorTransform>;
    fn color_transform_mut(&self, gc_context: MutationContext<'gc, '_>) -> RefMut<ColorTransform>;
    fn set_color_transform(
        &self,
        gc_context: MutationContext<'gc, '_>,
        color_transform: &ColorTransform,
    );

    /// Returns the matrix for transforming from this object's local space to global stage space.
    fn local_to_global_matrix(&self) -> Matrix {
        let mut node = self.parent();
        let mut matrix = *self.matrix();
        while let Some(display_object) = node {
            // TODO: We don't want to include the stage transform because it includes the scale
            // mode and alignment transform, but the AS APIs expect "global" to be relative to the
            // Stage, not final view coordinates.
            // I suspect we want this to include the stage transform eventually.
            if display_object.as_stage().is_some() {
                break;
            }
            matrix = *display_object.matrix() * matrix;
            node = display_object.parent();
        }
        matrix
    }

    /// Returns the matrix for transforming from global stage to this object's local space.
    fn global_to_local_matrix(&self) -> Matrix {
        let mut matrix = self.local_to_global_matrix();
        matrix.invert();
        matrix
    }

    /// Converts a local position to a global stage position
    fn local_to_global(&self, local: (Twips, Twips)) -> (Twips, Twips) {
        self.local_to_global_matrix() * local
    }

    /// Converts a local position on the stage to a local position on this display object
    fn global_to_local(&self, global: (Twips, Twips)) -> (Twips, Twips) {
        self.global_to_local_matrix() * global
    }

    /// The `x` position in pixels of this display object in local space.
    /// Returned by the `_x`/`x` ActionScript properties.
    fn x(&self) -> f64;

    /// Sets the `x` position in pixels of this display object in local space.
    /// Set by the `_x`/`x` ActionScript properties.
    fn set_x(&self, gc_context: MutationContext<'gc, '_>, value: f64);

    /// The `y` position in pixels of this display object in local space.
    /// Returned by the `_y`/`y` ActionScript properties.
    fn y(&self) -> f64;

    /// Sets the `y` position in pixels of this display object in local space.
    /// Set by the `_y`/`y` ActionScript properties.
    fn set_y(&self, gc_context: MutationContext<'gc, '_>, value: f64);

    /// The rotation in degrees this display object in local space.
    /// Returned by the `_rotation`/`rotation` ActionScript properties.
    fn rotation(&self, gc_context: MutationContext<'gc, '_>) -> Degrees;

    /// Sets the rotation in degrees this display object in local space.
    /// Set by the `_rotation`/`rotation` ActionScript properties.
    fn set_rotation(&self, gc_context: MutationContext<'gc, '_>, radians: Degrees);

    /// The X axis scale for this display object in local space.
    /// The normal scale is 100.
    /// Returned by the `_xscale`/`scaleX` ActionScript properties.
    fn scale_x(&self, gc_context: MutationContext<'gc, '_>) -> Percent;

    /// Sets the scale of the X axis for this display object in local space.
    /// The normal scale is 100.
    /// Set by the `_xscale`/`scaleX` ActionScript properties.
    fn set_scale_x(&self, gc_context: MutationContext<'gc, '_>, value: Percent);

    /// The Y axis scale for this display object in local space.
    /// The normal scale is 1.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    fn scale_y(&self, gc_context: MutationContext<'gc, '_>) -> Percent;

    /// Sets the Y axis scale for this display object in local space.
    /// The normal scale is 1.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    fn set_scale_y(&self, gc_context: MutationContext<'gc, '_>, value: Percent);

    /// Sets the pixel width of this display object in local space.
    /// The width is based on the AABB of the object.
    /// Returned by the ActionScript `_width`/`width` properties.
    fn width(&self) -> f64 {
        let bounds = self.local_bounds();
        (bounds.x_max.saturating_sub(bounds.x_min)).to_pixels()
    }

    /// Sets the pixel width of this display object in local space.
    /// The width is based on the AABB of the object.
    /// Set by the ActionScript `_width`/`width` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_width(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let object_bounds = self.bounds();
        let object_width = (object_bounds.x_max - object_bounds.x_min).to_pixels();
        let object_height = (object_bounds.y_max - object_bounds.y_min).to_pixels();
        let aspect_ratio = object_height / object_width;

        let (target_scale_x, target_scale_y) = if object_width != 0.0 {
            (value / object_width, value / object_height)
        } else {
            (0.0, 0.0)
        };

        // No idea about the derivation of this -- figured it out via lots of trial and error.
        // It has to do with the length of the sides A, B of an AABB enclosing the object's OBB with sides a, b:
        // A = sin(t) * a + cos(t) * b
        // B = cos(t) * a + sin(t) * b
        let prev_scale_x = self.scale_x(gc_context).into_unit();
        let prev_scale_y = self.scale_y(gc_context).into_unit();
        let rotation = self.rotation(gc_context);
        let cos = f64::abs(f64::cos(rotation.into_radians()));
        let sin = f64::abs(f64::sin(rotation.into_radians()));
        let mut new_scale_x = aspect_ratio * (cos * target_scale_x + sin * target_scale_y)
            / ((cos + aspect_ratio * sin) * (aspect_ratio * cos + sin));
        let mut new_scale_y =
            (sin * prev_scale_x + aspect_ratio * cos * prev_scale_y) / (aspect_ratio * cos + sin);

        if !new_scale_x.is_finite() {
            new_scale_x = 0.0;
        }

        if !new_scale_y.is_finite() {
            new_scale_y = 0.0;
        }

        self.set_scale_x(gc_context, Percent::from_unit(new_scale_x));
        self.set_scale_y(gc_context, Percent::from_unit(new_scale_y));
    }

    /// Gets the pixel height of the AABB containing this display object in local space.
    /// Returned by the ActionScript `_height`/`height` properties.
    fn height(&self) -> f64 {
        let bounds = self.local_bounds();
        (bounds.y_max.saturating_sub(bounds.y_min)).to_pixels()
    }

    /// Sets the pixel height of this display object in local space.
    /// Set by the ActionScript `_height`/`height` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_height(&self, gc_context: MutationContext<'gc, '_>, value: f64) {
        let object_bounds = self.bounds();
        let object_width = (object_bounds.x_max - object_bounds.x_min).to_pixels();
        let object_height = (object_bounds.y_max - object_bounds.y_min).to_pixels();
        let aspect_ratio = object_width / object_height;

        let (target_scale_x, target_scale_y) = if object_height != 0.0 {
            (value / object_width, value / object_height)
        } else {
            (0.0, 0.0)
        };

        // No idea about the derivation of this -- figured it out via lots of trial and error.
        // It has to do with the length of the sides A, B of an AABB enclosing the object's OBB with sides a, b:
        // A = sin(t) * a + cos(t) * b
        // B = cos(t) * a + sin(t) * b
        let prev_scale_x = self.scale_x(gc_context).into_unit();
        let prev_scale_y = self.scale_y(gc_context).into_unit();
        let rotation = self.rotation(gc_context);
        let cos = f64::abs(f64::cos(rotation.into_radians()));
        let sin = f64::abs(f64::sin(rotation.into_radians()));
        let mut new_scale_x =
            (aspect_ratio * cos * prev_scale_x + sin * prev_scale_y) / (aspect_ratio * cos + sin);
        let mut new_scale_y = aspect_ratio * (sin * target_scale_x + cos * target_scale_y)
            / ((cos + aspect_ratio * sin) * (aspect_ratio * cos + sin));

        if !new_scale_x.is_finite() {
            new_scale_x = 0.0;
        }

        if !new_scale_y.is_finite() {
            new_scale_y = 0.0;
        }

        self.set_scale_x(gc_context, Percent::from_unit(new_scale_x));
        self.set_scale_y(gc_context, Percent::from_unit(new_scale_y));
    }

    /// The opacity of this display object.
    /// 1 is fully opaque.
    /// Returned by the `_alpha`/`alpha` ActionScript properties.
    fn alpha(&self) -> f64;

    /// Sets the opacity of this display object.
    /// 1 is fully opaque.
    /// Set by the `_alpha`/`alpha` ActionScript properties.
    fn set_alpha(&self, gc_context: MutationContext<'gc, '_>, value: f64);

    fn name(&self) -> AvmString<'gc>;
    fn set_name(&self, gc_context: MutationContext<'gc, '_>, name: AvmString<'gc>);

    /// Returns the dot-syntax path to this display object, e.g. `_level0.foo.clip`
    fn path(&self) -> String {
        if let Some(parent) = self.avm1_parent() {
            let mut path = parent.path();
            path.push('.');
            path.push_str(&*self.name());
            path
        } else {
            format!("_level{}", self.depth())
        }
    }

    /// Returns the Flash 4 slash-syntax path to this display object, e.g. `/foo/clip`.
    /// Returned by the `_target` property in AVM1.
    fn slash_path(&self) -> String {
        fn build_slash_path(object: DisplayObject<'_>) -> String {
            if let Some(parent) = object.avm1_parent() {
                let mut path = build_slash_path(parent);
                path.push('/');
                path.push_str(&*object.name());
                path
            } else {
                let level = object.depth();
                if level == 0 {
                    // _level0 does not append its name in slash syntax.
                    String::new()
                } else {
                    // Other levels do append their name.
                    format!("_level{}", level)
                }
            }
        }

        if self.avm1_parent().is_some() {
            build_slash_path((*self).into())
        } else {
            // _target of _level0 should just be '/'.
            '/'.to_string()
        }
    }

    fn clip_depth(&self) -> Depth;
    fn set_clip_depth(&self, gc_context: MutationContext<'gc, '_>, depth: Depth);

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function merely exposes the display object parent,
    /// without any further filtering.
    fn parent(&self) -> Option<DisplayObject<'gc>>;

    /// Set the parent of this display object.
    fn set_parent(&self, gc_context: MutationContext<'gc, '_>, parent: Option<DisplayObject<'gc>>);

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function implements the concept of parenthood as
    /// seen in AVM1. Notably, it disallows access to the `Stage`; for an
    /// unfiltered concept of parent, use the `parent` method.
    fn avm1_parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent().filter(|p| p.as_stage().is_none())
    }

    /// Retrieve the parent of this display object.
    ///
    /// This version of the function implements the concept of parenthood as
    /// seen in AVM2. Notably, it disallows access to non-container parents.
    fn avm2_parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent().filter(|p| p.as_container().is_some())
    }

    fn prev_avm1_clip(&self) -> Option<DisplayObject<'gc>>;
    fn set_prev_avm1_clip(
        &self,
        gc_context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    );
    fn next_avm1_clip(&self) -> Option<DisplayObject<'gc>>;
    fn set_next_avm1_clip(
        &self,
        gc_context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    );
    fn masker(&self) -> Option<DisplayObject<'gc>>;
    fn set_masker(
        &self,
        gc_context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
        remove_old_link: bool,
    );
    fn maskee(&self) -> Option<DisplayObject<'gc>>;
    fn set_maskee(
        &self,
        gc_context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
        remove_old_link: bool,
    );

    fn removed(&self) -> bool;
    fn set_removed(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// Whether this display object is visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    fn visible(&self) -> bool;

    /// Sets whether this display object will be visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    fn set_visible(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// Whether this display object represents the root of loaded content.
    fn is_root(&self) -> bool;

    /// Sets whether this display object represents the root of loaded content.
    fn set_is_root(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// The sound transform for sounds played inside this display object.
    fn sound_transform(&self) -> Ref<SoundTransform>;

    /// The sound transform for sounds played inside this display object.
    fn set_sound_transform(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        sound_transform: SoundTransform,
    );

    /// Whether this display object is used as the _root of itself and its children.
    /// Returned by the `_lockroot` ActionScript property.
    fn lock_root(&self) -> bool;

    /// Sets whether this display object is used as the _root of itself and its children.
    /// Returned by the `_lockroot` ActionScript property.
    fn set_lock_root(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// Whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    fn transformed_by_script(&self) -> bool;

    /// Sets whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    fn set_transformed_by_script(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// Called whenever the focus tracker has deemed this display object worthy, or no longer worthy,
    /// of being the currently focused object.
    /// This should only be called by the focus manager. To change a focus, go through that.
    fn on_focus_changed(&self, _gc_context: MutationContext<'gc, '_>, _focused: bool) {}

    /// Whether or not this clip may be focusable for keyboard input.
    fn is_focusable(&self) -> bool {
        false
    }

    /// Whether this display object has been created by ActionScript 3.
    /// When this flag is set, changes from SWF `RemoveObject` tags are
    /// ignored.
    fn placed_by_script(&self) -> bool;

    /// Sets whether this display object has been created by ActionScript 3.
    /// When this flag is set, changes from SWF `RemoveObject` tags are
    /// ignored.
    fn set_placed_by_script(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// Whether this display object has been instantiated by the timeline.
    /// When this flag is set, attempts to change the object's name from AVM2
    /// throw an exception.
    fn instantiated_by_timeline(&self) -> bool;

    /// Sets whether this display object has been instantiated by the timeline.
    /// When this flag is set, attempts to change the object's name from AVM2
    /// throw an exception.
    fn set_instantiated_by_timeline(&self, gc_context: MutationContext<'gc, '_>, value: bool);

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed by its parent, and
    /// so forth.
    fn handle_clip_event(
        &self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        _event: ClipEvent,
    ) -> ClipEventResult {
        ClipEventResult::NotHandled
    }

    /// Emit an `enterFrame` event on this DisplayObject and any children it
    /// may have.
    fn enter_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut enter_frame_evt = Avm2Event::new("enterFrame");
        enter_frame_evt.set_bubbles(false);
        enter_frame_evt.set_cancelable(false);

        let dobject_constr = context.avm2.classes().display_object;

        if let Err(e) = Avm2::broadcast_event(context, enter_frame_evt, dobject_constr) {
            log::error!(
                "Encountered AVM2 error when broadcasting enterFrame event: {}",
                e
            );
        }
    }

    /// Construct all display objects that the timeline indicates should exist
    /// this frame, and their children.
    ///
    /// This function should ensure the following, from the point of view of
    /// downstream VMs:
    ///
    /// 1. That the object itself has been allocated, if not constructed
    /// 2. That newly created children have been instantiated and are present
    ///    as properties on the class
    fn construct_frame(&self, _context: &mut UpdateContext<'_, 'gc, '_>) {}

    /// Execute all other timeline actions on this object.
    fn run_frame(&self, _context: &mut UpdateContext<'_, 'gc, '_>) {}

    /// Execute all other timeline actions on this object and it's children.
    ///
    /// AVM2 operates recursively through children, so this also instructs
    /// children to run a frame.
    fn run_frame_avm2(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // Children run first.
        if let Some(container) = self.as_container() {
            for child in container.iter_render_list() {
                child.run_frame_avm2(context);
            }
        }

        self.run_frame(context);
    }

    /// Emit a `frameConstructed` event on this DisplayObject and any children it
    /// may have.
    fn frame_constructed(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut frame_constructed_evt = Avm2Event::new("frameConstructed");
        frame_constructed_evt.set_bubbles(false);
        frame_constructed_evt.set_cancelable(false);

        let dobject_constr = context.avm2.classes().display_object;

        if let Err(e) = Avm2::broadcast_event(context, frame_constructed_evt, dobject_constr) {
            log::error!(
                "Encountered AVM2 error when broadcasting frameConstructed event: {}",
                e
            );
        }
    }

    /// Run any frame scripts (if they exist and this object needs to run them).
    fn run_frame_scripts(self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if let Some(container) = self.as_container() {
            for child in container.iter_render_list() {
                child.run_frame_scripts(context);
            }
        }
    }

    /// Emit an `exitFrame` broadcast event.
    fn exit_frame(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let mut exit_frame_evt = Avm2Event::new("exitFrame");
        exit_frame_evt.set_bubbles(false);
        exit_frame_evt.set_cancelable(false);

        let dobject_constr = context.avm2.classes().display_object;

        if let Err(e) = Avm2::broadcast_event(context, exit_frame_evt, dobject_constr) {
            log::error!(
                "Encountered AVM2 error when broadcasting exitFrame event: {}",
                e
            );
        }
    }

    fn render_self(&self, _context: &mut RenderContext<'_, 'gc>) {}

    fn render(&self, context: &mut RenderContext<'_, 'gc>) {
        render_base((*self).into(), context)
    }

    fn unload(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        // Unload children.
        if let Some(ctr) = self.as_container() {
            for child in ctr.iter_render_list() {
                child.unload(context);
            }
        }

        if let Some(node) = self.maskee() {
            node.set_masker(context.gc_context, None, true);
        } else if let Some(node) = self.masker() {
            node.set_maskee(context.gc_context, None, true);
        }

        // Unregister any text field variable bindings, and replace them on the unbound list.
        if let Avm1Value::Object(object) = self.object() {
            if let Some(stage_object) = object.as_stage_object() {
                stage_object.unregister_text_field_bindings(context);
            }
        }

        self.set_removed(context.gc_context, true);
    }

    fn as_stage(&self) -> Option<Stage<'gc>> {
        None
    }
    fn as_avm1_button(&self) -> Option<Avm1Button<'gc>> {
        None
    }
    fn as_avm2_button(&self) -> Option<Avm2Button<'gc>> {
        None
    }
    fn as_movie_clip(&self) -> Option<MovieClip<'gc>> {
        None
    }
    fn as_edit_text(&self) -> Option<EditText<'gc>> {
        None
    }
    fn as_morph_shape(&self) -> Option<MorphShape<'gc>> {
        None
    }
    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        None
    }
    fn as_video(self) -> Option<Video<'gc>> {
        None
    }
    fn as_drawing(&self, _gc_context: MutationContext<'gc, '_>) -> Option<RefMut<'_, Drawing>> {
        None
    }
    fn as_bitmap(self) -> Option<Bitmap<'gc>> {
        None
    }
    fn as_interactive(self) -> Option<InteractiveObject<'gc>> {
        None
    }

    fn apply_place_object(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        placing_movie: Option<Arc<SwfMovie>>,
        place_object: &swf::PlaceObject,
    ) {
        // PlaceObject tags only apply if this object has not been dynamically moved by AS code.
        if !self.transformed_by_script() {
            if let Some(matrix) = place_object.matrix {
                self.set_matrix(context.gc_context, &matrix.into());
            }
            if let Some(color_transform) = &place_object.color_transform {
                self.set_color_transform(context.gc_context, &color_transform.clone().into());
            }
            if let Some(name) = &place_object.name {
                let encoding = swf::SwfStr::encoding_for_version(self.swf_version());
                let name = name.to_str_lossy(encoding).to_owned();
                self.set_name(context.gc_context, AvmString::new(context.gc_context, name));
            }
            if let Some(clip_depth) = place_object.clip_depth {
                self.set_clip_depth(context.gc_context, clip_depth.into());
            }
            if let Some(ratio) = place_object.ratio {
                if let Some(mut morph_shape) = self.as_morph_shape() {
                    morph_shape.set_ratio(context.gc_context, ratio);
                } else if let Some(video) = self.as_video() {
                    video.seek(context, ratio.into());
                }
            }
            // Clip events only apply to movie clips.
            if let (Some(clip_actions), Some(clip)) =
                (&place_object.clip_actions, self.as_movie_clip())
            {
                // Convert from `swf::ClipAction` to Ruffle's `ClipEventHandler`.
                use crate::display_object::movie_clip::ClipEventHandler;
                if let Some(placing_movie) = placing_movie {
                    clip.set_clip_event_handlers(
                        context.gc_context,
                        clip_actions
                            .iter()
                            .cloned()
                            .map(|a| {
                                ClipEventHandler::from_action_and_movie(
                                    a,
                                    Arc::clone(&placing_movie),
                                )
                            })
                            .collect(),
                    );
                } else {
                    // This probably shouldn't happen; we should always have a movie.
                    log::error!("No movie when trying to set clip event");
                }
            }
            if self.swf_version() >= 11 {
                if let Some(visible) = place_object.is_visible {
                    self.set_visible(context.gc_context, visible);
                }
            }
            // TODO: Others will go here eventually.
        }
    }

    /// Called when this object should be replaced by a PlaceObject tag.
    fn replace_with(&self, _context: &mut UpdateContext<'_, 'gc, '_>, _id: CharacterId) {
        // Noop for most symbols; only shapes can replace their innards with another Graphic.
    }

    fn object(&self) -> Avm1Value<'gc> {
        Avm1Value::Undefined // TODO: Implement for every type and delete this fallback.
    }

    fn object2(&self) -> Avm2Value<'gc> {
        Avm2Value::Undefined // TODO: See above.
    }

    fn set_object2(&mut self, _mc: MutationContext<'gc, '_>, _to: Avm2Object<'gc>) {}

    /// Tests if a given stage position point intersects with the world bounds of this object.
    fn hit_test_bounds(&self, pos: (Twips, Twips)) -> bool {
        self.world_bounds().contains(pos)
    }

    /// Tests if a given object's world bounds intersects with the world bounds
    /// of this object.
    fn hit_test_object(&self, other: DisplayObject<'gc>) -> bool {
        self.world_bounds().intersects(&other.world_bounds())
    }

    /// Tests if a given stage position point intersects within this object, considering the art.
    fn hit_test_shape(
        &self,
        _context: &mut UpdateContext<'_, 'gc, '_>,
        pos: (Twips, Twips),
        _options: HitTestOptions,
    ) -> bool {
        // Default to using bounding box.
        self.hit_test_bounds(pos)
    }

    #[allow(unused_variables)]
    fn mouse_pick(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        pos: (Twips, Twips),
        require_button_mode: bool,
    ) -> Option<DisplayObject<'gc>> {
        None
    }

    fn post_instantiation(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        _display_object: DisplayObject<'gc>,
        _init_object: Option<Avm1Object<'gc>>,
        _instantiated_by: Instantiator,
        run_frame: bool,
    ) {
        if run_frame {
            self.run_frame(context);
        }
    }

    /// Return the version of the SWF that created this movie clip.
    fn swf_version(&self) -> u8 {
        self.parent()
            .map(|p| p.swf_version())
            .unwrap_or(NEWEST_PLAYER_VERSION)
    }

    /// Return the SWF that defines this display object.
    fn movie(&self) -> Option<Arc<SwfMovie>> {
        self.parent().and_then(|p| p.movie())
    }

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc>;
    fn as_ptr(&self) -> *const DisplayObjectPtr;

    /// Whether this object can be used as a mask.
    /// If this returns false and this object is used as a mask, the mask will not be applied.
    /// This is used by movie clips to disable the mask when there are no children, for example.
    fn allow_as_mask(&self) -> bool {
        true
    }

    /// The cursor to use when this object is the hovered element under a mouse.
    fn mouse_cursor(&self) -> MouseCursor {
        MouseCursor::Hand
    }

    /// Obtain the top-most non-Stage parent of the display tree hierarchy, if
    /// a suitable object exists. If none such object exists, this function
    /// yields an AVM1 error (which shouldn't happen in normal usage).
    ///
    /// This function implements the AVM1 concept of root clips. For the AVM2
    /// version, see `avm2_root`.
    fn avm1_root(
        &self,
        context: &UpdateContext<'_, 'gc, '_>,
    ) -> Result<DisplayObject<'gc>, Avm1Error<'gc>> {
        let mut parent = if self.lock_root() {
            None
        } else {
            self.avm1_parent()
        };

        while let Some(p) = parent {
            if p.lock_root() {
                break;
            }

            let grandparent = p.avm1_parent();

            if grandparent.is_none() {
                break;
            }

            parent = grandparent;
        }

        parent
            .ok_or(Avm1Error::InvalidDisplayObjectHierarchy)
            .or_else(|_| {
                if let Avm1Value::Object(object) = self.object() {
                    object
                        .as_display_object()
                        .ok_or(Avm1Error::InvalidDisplayObjectHierarchy)
                } else if let Avm2Value::Object(object) = self.object2() {
                    if self.is_on_stage(context) {
                        object
                            .as_display_object()
                            .ok_or(Avm1Error::InvalidDisplayObjectHierarchy)
                    } else {
                        Err(Avm1Error::InvalidDisplayObjectHierarchy)
                    }
                } else {
                    Err(Avm1Error::InvalidDisplayObjectHierarchy)
                }
            })
    }

    /// Obtain the top-most non-Stage parent of the display tree hierarchy, if
    /// a suitable object exists.
    ///
    /// This function implements the AVM2 concept of root clips. For the AVM1
    /// version, see `avm1_root`.
    fn avm2_root(&self, _context: &mut UpdateContext<'_, 'gc, '_>) -> Option<DisplayObject<'gc>> {
        let mut parent = Some((*self).into());
        while let Some(p) = parent {
            if p.is_root() {
                return parent;
            }
            parent = p.parent();
        }
        None
    }

    /// Obtain the root of the display tree hierarchy, if a suitable object
    /// exists.
    ///
    /// This implements the AVM2 concept of `stage`. Notably, it deliberately
    /// will fail to locate the current player's stage for objects that are not
    /// rooted to the DisplayObject hierarchy correctly. If you just want to
    /// access the current player's stage, grab it from the context.
    fn avm2_stage(&self, _context: &UpdateContext<'_, 'gc, '_>) -> Option<DisplayObject<'gc>> {
        let mut parent = Some((*self).into());
        while let Some(p) = parent {
            if p.as_stage().is_some() {
                return parent;
            }
            parent = p.parent();
        }
        None
    }

    /// Determine if this display object is currently on the stage.
    fn is_on_stage(self, context: &UpdateContext<'_, 'gc, '_>) -> bool {
        let mut ancestor = self.avm2_parent();
        while let Some(parent) = ancestor {
            if parent.avm2_parent().is_some() {
                ancestor = parent.avm2_parent();
            } else {
                break;
            }
        }

        let ancestor = ancestor.unwrap_or_else(|| self.into());
        DisplayObject::ptr_eq(ancestor, context.stage.into())
    }

    /// Assigns a default instance name `instanceN` to this object.
    fn set_default_instance_name(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        if self.name().is_empty() {
            let name = format!("instance{}", *context.instance_counter);
            self.set_name(context.gc_context, AvmString::new(context.gc_context, name));
            *context.instance_counter = context.instance_counter.wrapping_add(1);
        }
    }

    /// Assigns a default root name to this object.
    ///
    /// The default root names change based on the AVM configuration of the
    /// clip; AVM2 clips get `rootN` while AVM1 clips get blank strings.
    fn set_default_root_name(&self, context: &mut UpdateContext<'_, 'gc, '_>) {
        let vm_type = context.avm_type();

        if matches!(vm_type, AvmType::Avm2) {
            let name = AvmString::new(context.gc_context, format!("root{}", self.depth() + 1));
            self.set_name(context.gc_context, name);
        } else if matches!(vm_type, AvmType::Avm1) {
            self.set_name(context.gc_context, Default::default());
        }
    }

    fn bind_text_field_variables(&self, activation: &mut Activation<'_, 'gc, '_>) {
        // Check all unbound text fields to see if they apply to this object.
        // TODO: Replace with `Vec::drain_filter` when stable.
        let mut i = 0;
        let mut len = activation.context.unbound_text_fields.len();
        while i < len {
            if activation.context.unbound_text_fields[i]
                .try_bind_text_field_variable(activation, false)
            {
                activation.context.unbound_text_fields.swap_remove(i);
                len -= 1;
            } else {
                i += 1;
            }
        }
    }
}

pub enum DisplayObjectPtr {}

// To use this macro: `use crate::impl_display_object_sansbounds;` or `use crate::prelude::*;`
#[macro_export]
macro_rules! impl_display_object_sansbounds {
    ($field:ident) => {
        fn depth(&self) -> crate::prelude::Depth {
            self.0.read().$field.depth()
        }
        fn set_depth(&self, gc_context: gc_arena::MutationContext<'gc, '_>, depth: Depth) {
            self.0.write(gc_context).$field.set_depth(depth)
        }
        fn place_frame(&self) -> u16 {
            self.0.read().$field.place_frame()
        }
        fn set_place_frame(&self, context: gc_arena::MutationContext<'gc, '_>, frame: u16) {
            self.0.write(context).$field.set_place_frame(frame)
        }
        fn transform(&self) -> std::cell::Ref<crate::transform::Transform> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.transform())
        }
        fn matrix(&self) -> std::cell::Ref<crate::matrix::Matrix> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.matrix())
        }
        fn matrix_mut(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
        ) -> std::cell::RefMut<crate::matrix::Matrix> {
            std::cell::RefMut::map(self.0.write(context), |o| o.$field.matrix_mut())
        }
        fn color_transform(&self) -> std::cell::Ref<crate::color_transform::ColorTransform> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.color_transform())
        }
        fn color_transform_mut(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
        ) -> std::cell::RefMut<crate::color_transform::ColorTransform> {
            std::cell::RefMut::map(self.0.write(context), |o| o.$field.color_transform_mut())
        }
        fn set_color_transform(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            color_transform: &crate::color_transform::ColorTransform,
        ) {
            self.0
                .write(context)
                .$field
                .set_color_transform(color_transform)
        }
        fn rotation(&self, gc_context: gc_arena::MutationContext<'gc, '_>) -> Degrees {
            self.0.write(gc_context).$field.rotation()
        }
        fn set_rotation(&self, gc_context: gc_arena::MutationContext<'gc, '_>, degrees: Degrees) {
            self.0.write(gc_context).$field.set_rotation(degrees)
        }
        fn scale_x(&self, gc_context: gc_arena::MutationContext<'gc, '_>) -> Percent {
            self.0.write(gc_context).$field.scale_x()
        }
        fn set_scale_x(&self, gc_context: gc_arena::MutationContext<'gc, '_>, value: Percent) {
            self.0.write(gc_context).$field.set_scale_x(value)
        }
        fn scale_y(&self, gc_context: gc_arena::MutationContext<'gc, '_>) -> Percent {
            self.0.write(gc_context).$field.scale_y()
        }
        fn set_scale_y(&self, gc_context: gc_arena::MutationContext<'gc, '_>, value: Percent) {
            self.0.write(gc_context).$field.set_scale_y(value)
        }
        fn alpha(&self) -> f64 {
            self.0.read().$field.alpha()
        }
        fn set_alpha(&self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_alpha(value)
        }
        fn name(&self) -> crate::string::AvmString<'gc> {
            self.0.read().$field.name()
        }
        fn set_name(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            name: crate::string::AvmString<'gc>,
        ) {
            self.0.write(context).$field.set_name(name)
        }
        fn clip_depth(&self) -> crate::prelude::Depth {
            self.0.read().$field.clip_depth()
        }
        fn set_clip_depth(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            depth: crate::prelude::Depth,
        ) {
            self.0.write(context).$field.set_clip_depth(depth)
        }
        fn parent(&self) -> Option<crate::display_object::DisplayObject<'gc>> {
            self.0.read().$field.parent()
        }
        fn set_parent(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            parent: Option<crate::display_object::DisplayObject<'gc>>,
        ) {
            self.0.write(context).$field.set_parent(parent)
        }
        fn prev_avm1_clip(&self) -> Option<crate::display_object::DisplayObject<'gc>> {
            self.0.read().$field.prev_avm1_clip()
        }
        fn set_prev_avm1_clip(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            node: Option<crate::display_object::DisplayObject<'gc>>,
        ) {
            self.0.write(context).$field.set_prev_avm1_clip(node);
        }
        fn next_avm1_clip(&self) -> Option<crate::display_object::DisplayObject<'gc>> {
            self.0.read().$field.next_avm1_clip()
        }
        fn set_next_avm1_clip(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            node: Option<crate::display_object::DisplayObject<'gc>>,
        ) {
            self.0.write(context).$field.set_next_avm1_clip(node);
        }
        fn masker(&self) -> Option<DisplayObject<'gc>> {
            self.0.read().$field.masker()
        }
        fn set_masker(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            node: Option<DisplayObject<'gc>>,
            remove_old_link: bool,
        ) {
            if remove_old_link {
                if let Some(old_masker) = self.0.read().$field.masker() {
                    old_masker.set_maskee(context, None, false);
                }
            }
            self.0.write(context).$field.set_masker(node);
        }
        fn maskee(&self) -> Option<DisplayObject<'gc>> {
            self.0.read().$field.maskee()
        }
        fn set_maskee(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            node: Option<DisplayObject<'gc>>,
            remove_old_link: bool,
        ) {
            if remove_old_link {
                if let Some(old_maskee) = self.0.read().$field.maskee() {
                    old_maskee.set_masker(context, None, false);
                }
            }
            self.0.write(context).$field.set_maskee(node);
        }
        fn removed(&self) -> bool {
            self.0.read().$field.removed()
        }
        fn set_removed(&self, context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_removed(value)
        }
        fn sound_transform(&self) -> std::cell::Ref<crate::display_object::SoundTransform> {
            std::cell::Ref::map(self.0.read(), |r| r.$field.sound_transform())
        }
        fn set_sound_transform(
            &self,
            context: &mut crate::context::UpdateContext<'_, 'gc, '_>,
            value: crate::display_object::SoundTransform,
        ) {
            self.0
                .write(context.gc_context)
                .$field
                .set_sound_transform(value);
            context.set_sound_transforms_dirty();
        }
        fn visible(&self) -> bool {
            self.0.read().$field.visible()
        }
        fn set_visible(&self, context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_visible(value);
        }
        fn is_root(&self) -> bool {
            self.0.read().$field.is_root()
        }
        fn set_is_root(&self, context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_is_root(value);
        }
        fn lock_root(&self) -> bool {
            self.0.read().$field.lock_root()
        }
        fn set_lock_root(&self, context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_lock_root(value);
        }
        fn transformed_by_script(&self) -> bool {
            self.0.read().$field.transformed_by_script()
        }
        fn set_transformed_by_script(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            value: bool,
        ) {
            self.0
                .write(context)
                .$field
                .set_transformed_by_script(value)
        }
        fn placed_by_script(&self) -> bool {
            self.0.read().$field.placed_by_script()
        }
        fn set_placed_by_script(&self, context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_placed_by_script(value)
        }
        fn instantiated_by_timeline(&self) -> bool {
            self.0.read().$field.instantiated_by_timeline()
        }
        fn set_instantiated_by_timeline(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            value: bool,
        ) {
            self.0
                .write(context)
                .$field
                .set_instantiated_by_timeline(value)
        }
        fn instantiate(
            &self,
            gc_context: gc_arena::MutationContext<'gc, '_>,
        ) -> crate::display_object::DisplayObject<'gc> {
            Self(gc_arena::GcCell::allocate(
                gc_context,
                self.0.read().clone(),
            ))
            .into()
        }
        fn as_ptr(&self) -> *const crate::display_object::DisplayObjectPtr {
            self.0.as_ptr() as *const crate::display_object::DisplayObjectPtr
        }
    };
}

// To use this macro: `use crate::impl_display_object;` or `use crate::prelude::*;`
#[macro_export]
macro_rules! impl_display_object {
    ($field:ident) => {
        impl_display_object_sansbounds!($field);

        fn x(&self) -> f64 {
            self.0.read().$field.x()
        }
        fn set_x(&self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_x(value)
        }
        fn y(&self) -> f64 {
            self.0.read().$field.y()
        }
        fn set_y(&self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_y(value)
        }
        fn set_matrix(
            &self,
            context: gc_arena::MutationContext<'gc, '_>,
            matrix: &crate::matrix::Matrix,
        ) {
            self.0.write(context).$field.set_matrix(matrix)
        }
    };
}

impl<'gc> DisplayObject<'gc> {
    pub fn ptr_eq(a: DisplayObject<'gc>, b: DisplayObject<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }

    pub fn option_ptr_eq(a: Option<DisplayObject<'gc>>, b: Option<DisplayObject<'gc>>) -> bool {
        a.map(|o| o.as_ptr()) == b.map(|o| o.as_ptr())
    }
}

bitflags! {
    /// Bit flags used by `DisplayObject`.
    #[derive(Collect)]
    #[collect(no_drop)]
    struct DisplayObjectFlags: u8 {
        /// Whether this object has been removed from the display list.
        /// Necessary in AVM1 to throw away queued actions from removed movie clips.
        const REMOVED                  = 1 << 0;

        /// If this object is visible (`_visible` property).
        const VISIBLE                  = 1 << 1;

        /// Whether the `_xscale`, `_yscale` and `_rotation` of the object have been calculated and cached.
        const SCALE_ROTATION_CACHED    = 1 << 2;

        /// Whether this object has been transformed by ActionScript.
        /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
        const TRANSFORMED_BY_SCRIPT    = 1 << 3;

        /// Whether this object has been placed in a container by ActionScript 3.
        /// When this flag is set, changes from SWF `RemoveObject` tags are ignored.
        const PLACED_BY_SCRIPT         = 1 << 4;

        /// Whether this object has been instantiated by a SWF tag.
        /// When this flag is set, changes from SWF `RemoveObject` tags are ignored.
        const INSTANTIATED_BY_TIMELINE = 1 << 5;

        /// Whether this object is a "root", the top-most display object of a loaded SWF or Bitmap.
        /// Used by `MovieClip.getBytesLoaded` in AVM1 and `DisplayObject.root` in AVM2.
        const IS_ROOT                  = 1 << 6;

        /// Whether this object has `_lockroot` set to true, in which case
        /// it becomes the _root of itself and of any children
        const LOCK_ROOT                = 1 << 7;
    }
}

bitflags! {
    /// Defines how hit testing should be performed.
    /// Used for mouse picking and ActionScript's hitTestClip functions.
    pub struct HitTestOptions: u8 {
        /// Ignore objects used as masks (setMask / clipDepth).
        const SKIP_MASK = 1 << 0;

        /// Ignore objects with the ActionScript's visibility flag turned off.
        const SKIP_INVISIBLE = 1 << 1;

        /// The options used for `hitTest` calls in ActionScript.
        const AVM_HIT_TEST = Self::SKIP_MASK.bits;

        /// The options used for mouse picking, such as clicking on buttons.
        const MOUSE_PICK = Self::SKIP_MASK.bits | Self::SKIP_INVISIBLE.bits;
    }
}

/// Represents the sound transform of sounds played inside a Flash MovieClip.
/// Every value is a percentage (0-100), but out of range values are allowed.
/// In AVM1, this is returned by `Sound.getTransform`.
/// In AVM2, this is returned by `Sprite.soundTransform`.
#[derive(Debug, PartialEq, Eq, Clone, Collect)]
#[collect(require_static)]
pub struct SoundTransform {
    pub volume: i32,
    pub left_to_left: i32,
    pub left_to_right: i32,
    pub right_to_left: i32,
    pub right_to_right: i32,
}

impl SoundTransform {
    pub const MAX_VOLUME: i32 = 100;

    /// Applies another SoundTransform on top of this SoundTransform.
    pub fn concat(&mut self, other: &SoundTransform) {
        // This is a 2x2 matrix multiply between the transforms.
        // Done with integer math to match Flash behavior.
        const MAX_VOLUME: i64 = SoundTransform::MAX_VOLUME as i64;
        self.volume = (i64::from(self.volume) * i64::from(other.volume) / MAX_VOLUME) as i32;

        let ll0: i64 = self.left_to_left.into();
        let lr0: i64 = self.left_to_right.into();
        let rl0: i64 = self.right_to_left.into();
        let rr0: i64 = self.right_to_right.into();
        let ll1: i64 = other.left_to_left.into();
        let lr1: i64 = other.left_to_right.into();
        let rl1: i64 = other.right_to_left.into();
        let rr1: i64 = other.right_to_right.into();
        self.left_to_left = ((ll0 * ll1 + rl0 * lr1) / MAX_VOLUME) as i32;
        self.left_to_right = ((lr0 * ll1 + rr0 * lr1) / MAX_VOLUME) as i32;
        self.right_to_left = ((ll0 * rl1 + rl0 * rr1) / MAX_VOLUME) as i32;
        self.right_to_right = ((lr0 * rl1 + rr0 * rr1) / MAX_VOLUME) as i32;
    }

    /// Returns the pan of this transform.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.getPan()`
    pub fn pan(&self) -> i32 {
        // It's not clear why Flash has the weird `abs` behavior, but this
        // mathes the values that Flash returns (see `sound` regression test).
        if self.left_to_left != Self::MAX_VOLUME {
            Self::MAX_VOLUME - self.left_to_left.abs()
        } else {
            self.right_to_right.abs() - Self::MAX_VOLUME
        }
    }

    /// Sets this transform of this pan.
    /// -100 is full left and 100 is full right.
    /// This matches the behavior of AVM1 `Sound.setPan()`.
    pub fn set_pan(&mut self, pan: i32) {
        if pan >= 0 {
            self.left_to_left = Self::MAX_VOLUME - pan;
            self.right_to_right = Self::MAX_VOLUME;
        } else {
            self.left_to_left = Self::MAX_VOLUME;
            self.right_to_right = Self::MAX_VOLUME + pan;
        }
        self.left_to_right = 0;
        self.right_to_left = 0;
    }

    pub fn from_avm2_object<'gc>(
        activation: &mut Avm2Activation<'_, 'gc, '_>,
        as3_st: Avm2Object<'gc>,
    ) -> Result<Self, Avm2Error> {
        Ok(SoundTransform {
            left_to_left: (as3_st
                .get_property(
                    as3_st,
                    &Avm2QName::new(Avm2Namespace::public(), "leftToLeft").into(),
                    activation,
                )?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            left_to_right: (as3_st
                .get_property(
                    as3_st,
                    &Avm2QName::new(Avm2Namespace::public(), "leftToRight").into(),
                    activation,
                )?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            right_to_left: (as3_st
                .get_property(
                    as3_st,
                    &Avm2QName::new(Avm2Namespace::public(), "rightToLeft").into(),
                    activation,
                )?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            right_to_right: (as3_st
                .get_property(
                    as3_st,
                    &Avm2QName::new(Avm2Namespace::public(), "rightToRight").into(),
                    activation,
                )?
                .coerce_to_number(activation)?
                * 100.0) as i32,
            volume: (as3_st
                .get_property(
                    as3_st,
                    &Avm2QName::new(Avm2Namespace::public(), "volume").into(),
                    activation,
                )?
                .coerce_to_number(activation)?
                * 100.0) as i32,
        })
    }

    pub fn into_avm2_object<'gc>(
        self,
        activation: &mut Avm2Activation<'_, 'gc, '_>,
    ) -> Result<Avm2Object<'gc>, Avm2Error> {
        let mut as3_st = activation
            .avm2()
            .classes()
            .soundtransform
            .construct(activation, &[])?;

        as3_st.set_property(
            as3_st,
            &Avm2QName::new(Avm2Namespace::public(), "leftToLeft").into(),
            (self.left_to_left as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_property(
            as3_st,
            &Avm2QName::new(Avm2Namespace::public(), "leftToRight").into(),
            (self.left_to_right as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_property(
            as3_st,
            &Avm2QName::new(Avm2Namespace::public(), "rightToLeft").into(),
            (self.right_to_left as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_property(
            as3_st,
            &Avm2QName::new(Avm2Namespace::public(), "rightToRight").into(),
            (self.right_to_right as f64 / 100.0).into(),
            activation,
        )?;
        as3_st.set_property(
            as3_st,
            &Avm2QName::new(Avm2Namespace::public(), "volume").into(),
            (self.volume as f64 / 100.0).into(),
            activation,
        )?;

        Ok(as3_st)
    }
}

impl Default for SoundTransform {
    fn default() -> Self {
        Self {
            volume: 100,
            left_to_left: 100,
            left_to_right: 0,
            right_to_left: 0,
            right_to_right: 100,
        }
    }
}
