use crate::avm1::{Object, TObject, Value};
use crate::context::{RenderContext, UpdateContext};
use crate::player::NEWEST_PLAYER_VERSION;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::transform::Transform;
use enumset::{EnumSet, EnumSetType};
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
use std::cmp::min;
use std::fmt::Debug;
use std::sync::Arc;

mod bitmap;
mod button;
mod edit_text;
mod graphic;
mod morph_shape;
mod movie_clip;
mod text;

use crate::events::{ButtonEvent, ButtonEventResult, ClipEvent};
pub use bitmap::Bitmap;
pub use button::Button;
pub use edit_text::EditText;
pub use graphic::Graphic;
pub use morph_shape::{MorphShape, MorphShapeStatic};
pub use movie_clip::MovieClip;
pub use text::Text;

#[derive(Clone, Debug)]
pub struct DisplayObjectBase<'gc> {
    parent: Option<DisplayObject<'gc>>,
    place_frame: u16,
    depth: Depth,
    transform: Transform,
    name: String,
    clip_depth: Depth,

    // Cached transform properties `_xscale`, `_yscale`, `_rotation`.
    // These are expensive to calculate, so they will be calculated and cached when AS requests
    // one of these properties.
    rotation: f64,
    scale_x: f64,
    scale_y: f64,
    skew: f64,

    /// The first child of this display object in order of execution.
    /// This is differen than render order.
    first_child: Option<DisplayObject<'gc>>,

    /// The previous sibling of this display object in order of execution.
    prev_sibling: Option<DisplayObject<'gc>>,

    /// The next sibling of this display object in order of execution.
    next_sibling: Option<DisplayObject<'gc>>,

    /// Bit flags for various display object properites.
    flags: EnumSet<DisplayObjectFlags>,
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
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            skew: 0.0,
            first_child: None,
            prev_sibling: None,
            next_sibling: None,
            flags: DisplayObjectFlags::Visible.into(),
        }
    }
}

unsafe impl<'gc> Collect for DisplayObjectBase<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.parent.trace(cc);
        self.first_child.trace(cc);
        self.prev_sibling.trace(cc);
        self.next_sibling.trace(cc);
    }
}

#[allow(dead_code)]
impl<'gc> DisplayObjectBase<'gc> {
    /// Reset all properties that would be adjusted by a movie load.
    fn reset_for_movie_load(&mut self) {
        self.first_child = None;
        self.flags = DisplayObjectFlags::Visible.into();
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
    fn set_place_frame(&mut self, _context: MutationContext<'gc, '_>, frame: u16) {
        self.place_frame = frame;
    }
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn matrix(&self) -> &Matrix {
        &self.transform.matrix
    }
    fn matrix_mut(&mut self, _context: MutationContext<'gc, '_>) -> &mut Matrix {
        &mut self.transform.matrix
    }
    fn set_matrix(&mut self, _context: MutationContext<'gc, '_>, matrix: &Matrix) {
        self.transform.matrix = *matrix;
        self.flags.remove(DisplayObjectFlags::ScaleRotationCached);
    }
    fn color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }
    fn color_transform_mut(&mut self) -> &mut ColorTransform {
        &mut self.transform.color_transform
    }
    fn set_color_transform(
        &mut self,
        _context: MutationContext<'gc, '_>,
        color_transform: &ColorTransform,
    ) {
        self.transform.color_transform = *color_transform;
    }
    fn x(&self) -> f64 {
        f64::from(self.transform.matrix.tx) / Twips::TWIPS_PER_PIXEL
    }
    fn set_x(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.transform.matrix.tx = (value * Twips::TWIPS_PER_PIXEL) as f32
    }
    fn y(&self) -> f64 {
        f64::from(self.transform.matrix.ty) / Twips::TWIPS_PER_PIXEL
    }
    fn set_y(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.transform.matrix.ty = (value * Twips::TWIPS_PER_PIXEL) as f32
    }

    /// Caches the scale and rotation factors for this display object, if necessary.
    /// Calculating these requires heavy trig ops, so we only do it when `_xscale`, `_yscale` or
    /// `_rotation` is accessed.
    fn cache_scale_rotation(&mut self) {
        if !self.flags.contains(DisplayObjectFlags::ScaleRotationCached) {
            let (a, b, c, d) = (
                self.transform.matrix.a,
                self.transform.matrix.b,
                self.transform.matrix.c,
                self.transform.matrix.d,
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
            let rotation_x = f32::atan2(b, a);
            let rotation_y = f32::atan2(-c, d);
            let scale_x = f32::sqrt(a * a + b * b);
            let scale_y = f32::sqrt(c * c + d * d);
            self.rotation = rotation_x.into();
            self.scale_x = scale_x.into();
            self.scale_y = scale_y.into();
            self.skew = (rotation_y - rotation_x).into();
            self.flags.insert(DisplayObjectFlags::ScaleRotationCached);
        }
    }

    fn set_scale(&mut self, scale_x: f32, scale_y: f32, rotation: f32) {
        self.cache_scale_rotation();
        let mut matrix = &mut self.transform.matrix;
        let rotation = rotation.to_radians();
        let cos_x = f32::cos(rotation);
        let sin_x = f32::sin(rotation);
        self.scale_x = scale_x.into();
        self.scale_y = scale_y.into();
        self.rotation = rotation.into();
        matrix.a = (scale_x * cos_x) as f32;
        matrix.b = (scale_x * sin_x) as f32;
        matrix.c = (scale_y * -sin_x) as f32;
        matrix.d = (scale_y * cos_x) as f32;
    }

    fn rotation(&mut self) -> f64 {
        self.cache_scale_rotation();
        self.rotation
    }
    fn set_rotation(&mut self, radians: f64) {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.rotation = radians;
        let cos_x = f64::cos(radians);
        let sin_x = f64::sin(radians);
        let cos_y = f64::cos(radians + self.skew);
        let sin_y = f64::sin(radians + self.skew);
        let mut matrix = &mut self.transform.matrix;
        matrix.a = (self.scale_x * cos_x) as f32;
        matrix.b = (self.scale_x * sin_x) as f32;
        matrix.c = (self.scale_y * -sin_y) as f32;
        matrix.d = (self.scale_y * cos_y) as f32;
    }
    fn scale_x(&mut self) -> f64 {
        self.cache_scale_rotation();
        self.scale_x
    }
    fn set_scale_x(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_x = value;
        let cos = f64::cos(self.rotation);
        let sin = f64::sin(self.rotation);
        let mut matrix = &mut self.transform.matrix;
        matrix.a = (cos * value) as f32;
        matrix.b = (sin * value) as f32;
    }
    fn scale_y(&mut self) -> f64 {
        self.cache_scale_rotation();
        self.scale_y
    }
    fn set_scale_y(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.cache_scale_rotation();
        self.scale_y = value;
        let cos = f64::cos(self.rotation + self.skew);
        let sin = f64::sin(self.rotation + self.skew);
        let mut matrix = &mut self.transform.matrix;
        matrix.c = (-sin * value) as f32;
        matrix.d = (cos * value) as f32;
    }

    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, _context: MutationContext<'gc, '_>, name: &str) {
        self.name = name.to_string();
    }
    fn alpha(&self) -> f64 {
        f64::from(self.color_transform().a_mult)
    }
    fn set_alpha(&mut self, value: f64) {
        self.set_transformed_by_script(true);
        self.color_transform_mut().a_mult = value as f32
    }
    fn clip_depth(&self) -> Depth {
        self.clip_depth
    }
    fn set_clip_depth(&mut self, _context: MutationContext<'gc, '_>, depth: Depth) {
        self.clip_depth = depth;
    }
    fn parent(&self) -> Option<DisplayObject<'gc>> {
        self.parent
    }
    fn set_parent(
        &mut self,
        _context: MutationContext<'gc, '_>,
        parent: Option<DisplayObject<'gc>>,
    ) {
        self.parent = parent;
    }
    fn first_child(&self) -> Option<DisplayObject<'gc>> {
        self.first_child
    }
    fn set_first_child(
        &mut self,
        _context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    ) {
        self.first_child = node;
    }
    fn prev_sibling(&self) -> Option<DisplayObject<'gc>> {
        self.prev_sibling
    }
    fn set_prev_sibling(
        &mut self,
        _context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    ) {
        self.prev_sibling = node;
    }
    fn next_sibling(&self) -> Option<DisplayObject<'gc>> {
        self.next_sibling
    }
    fn set_next_sibling(
        &mut self,
        _context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    ) {
        self.next_sibling = node;
    }
    fn removed(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::Removed)
    }
    fn set_removed(&mut self, value: bool) {
        if value {
            self.flags.insert(DisplayObjectFlags::Removed);
        } else {
            self.flags.remove(DisplayObjectFlags::Removed);
        }
    }

    fn visible(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::Visible)
    }

    fn set_visible(&mut self, value: bool) {
        if value {
            self.flags.insert(DisplayObjectFlags::Visible);
        } else {
            self.flags.remove(DisplayObjectFlags::Visible);
        }
    }

    fn transformed_by_script(&self) -> bool {
        self.flags.contains(DisplayObjectFlags::TransformedByScript)
    }

    fn set_transformed_by_script(&mut self, value: bool) {
        if value {
            self.flags.insert(DisplayObjectFlags::TransformedByScript);
        } else {
            self.flags.remove(DisplayObjectFlags::TransformedByScript);
        }
    }

    fn swf_version(&self) -> u8 {
        self.parent
            .map(|p| p.swf_version())
            .unwrap_or(NEWEST_PLAYER_VERSION)
    }

    fn movie(&self) -> Option<Arc<SwfMovie>> {
        self.parent.and_then(|p| p.movie())
    }
}

#[enum_trait_object(
    #[derive(Clone, Collect, Debug, Copy)]
    #[collect(no_drop)]
    pub enum DisplayObject<'gc> {
        Bitmap(Bitmap<'gc>),
        Button(Button<'gc>),
        EditText(EditText<'gc>),
        Graphic(Graphic<'gc>),
        MorphShape(MorphShape<'gc>),
        MovieClip(MovieClip<'gc>),
        Text(Text<'gc>),
    }
)]
pub trait TDisplayObject<'gc>: 'gc + Collect + Debug + Into<DisplayObject<'gc>> {
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
        for child in self.children() {
            let matrix = *matrix * *child.matrix();
            bounds.union(&child.bounds_with_transform(&matrix));
        }
        bounds
    }

    fn place_frame(&self) -> u16;
    fn set_place_frame(&mut self, context: MutationContext<'gc, '_>, frame: u16);

    fn transform(&self) -> Ref<Transform>;
    fn matrix(&self) -> Ref<Matrix>;
    fn matrix_mut(&mut self, context: MutationContext<'gc, '_>) -> RefMut<Matrix>;
    fn set_matrix(&mut self, context: MutationContext<'gc, '_>, matrix: &Matrix);
    fn color_transform(&self) -> Ref<ColorTransform>;
    fn color_transform_mut(&self, context: MutationContext<'gc, '_>) -> RefMut<ColorTransform>;
    fn set_color_transform(
        &mut self,
        context: MutationContext<'gc, '_>,
        color_transform: &ColorTransform,
    );

    /// Returns the matrix for transforming from this object's local space to global stage space.
    fn local_to_global_matrix(&self) -> Matrix {
        let mut node = self.parent();
        let mut matrix = *self.matrix();
        while let Some(display_object) = node {
            matrix = *display_object.matrix() * matrix;
            node = display_object.parent();
        }

        matrix
    }

    /// Returns the matrix for transforming from global stage to this object's local space.
    fn global_to_local_matrix(&self) -> Matrix {
        let mut node = self.parent();
        let mut matrix = *self.matrix();
        while let Some(display_object) = node {
            matrix = *display_object.matrix() * matrix;
            node = display_object.parent();
        }

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
    fn set_x(&mut self, gc_context: MutationContext<'gc, '_>, value: f64);

    /// The `y` position in pixels of this display object in local space.
    /// Returned by the `_y`/`y` ActionScript properties.
    fn y(&self) -> f64;

    /// Sets the `y` position in pixels of this display object in local space.
    /// Set by the `_y`/`y` ActionScript properties.
    fn set_y(&mut self, gc_context: MutationContext<'gc, '_>, value: f64);

    /// The rotation in radians this display object in local space.
    /// Returned by the `_rotation`/`rotation` ActionScript properties.
    /// Note that the ActionScript properties return the angle in degrees;
    /// the conversion to degrees is done in the AVM.
    fn rotation(&mut self, gc_context: MutationContext<'gc, '_>) -> f64;

    /// Sets the rotation in radians this display object in local space.
    /// Set by the `_rotation`/`rotation` ActionScript properties.
    /// Note that the ActionScript properties set the angle in degrees;
    /// the conversion to radians is done in the AVM.
    fn set_rotation(&mut self, gc_context: MutationContext<'gc, '_>, radians: f64);

    /// The X axis scale for this display object in local space.
    /// The normal scale is 1.
    /// Returned by the `_xscale`/`scaleX` ActionScript properties.
    fn scale_x(&mut self, gc_context: MutationContext<'gc, '_>) -> f64;

    /// Sets the scale of the X axis for this display object in local space.
    /// The normal scale is 1.
    /// Set by the `_xscale`/`scaleX` ActionScript properties.
    fn set_scale_x(&mut self, gc_context: MutationContext<'gc, '_>, value: f64);

    /// The Y axis scale for this display object in local space.
    /// The normal scale is 1.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    fn scale_y(&mut self, gc_context: MutationContext<'gc, '_>) -> f64;

    /// Sets the Y axis scale for this display object in local space.
    /// The normal scale is 1.
    /// Returned by the `_yscale`/`scaleY` ActionScript properties.
    fn set_scale_y(&mut self, gc_context: MutationContext<'gc, '_>, value: f64);

    /// Sets the pixel width of this display object in local space.
    /// The width is based on the AABB of the object.
    /// Returned by the ActionScript `_width`/`width` properties.
    fn width(&self) -> f64 {
        let bounds = self.local_bounds();
        (bounds.x_max - bounds.x_min).to_pixels()
    }

    /// Sets the pixel width of this display object in local space.
    /// The width is based on the AABB of the object.
    /// Set by the ActionScript `_width`/`width` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_width(&mut self, gc_context: MutationContext<'gc, '_>, value: f64) {
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
        let prev_scale_x = self.scale_x(gc_context);
        let prev_scale_y = self.scale_y(gc_context);
        let rotation = self.rotation(gc_context);
        let cos = f64::abs(f64::cos(rotation));
        let sin = f64::abs(f64::sin(rotation));
        let new_scale_x = aspect_ratio * (cos * target_scale_x + sin * target_scale_y)
            / ((cos + aspect_ratio * sin) * (aspect_ratio * cos + sin));
        let new_scale_y =
            (sin * prev_scale_x + aspect_ratio * cos * prev_scale_y) / (aspect_ratio * cos + sin);
        self.set_scale_x(gc_context, new_scale_x);
        self.set_scale_y(gc_context, new_scale_y);
    }
    /// Gets the pixel height of the AABB containing this display object in local space.
    /// Returned by the ActionScript `_height`/`height` properties.
    fn height(&self) -> f64 {
        let bounds = self.local_bounds();
        (bounds.y_max - bounds.y_min).to_pixels()
    }
    /// Sets the pixel height of this display object in local space.
    /// Set by the ActionScript `_height`/`height` properties.
    /// This does odd things on rotated clips to match the behavior of Flash.
    fn set_height(&mut self, gc_context: MutationContext<'gc, '_>, value: f64) {
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
        let prev_scale_x = self.scale_x(gc_context);
        let prev_scale_y = self.scale_y(gc_context);
        let rotation = self.rotation(gc_context);
        let cos = f64::abs(f64::cos(rotation));
        let sin = f64::abs(f64::sin(rotation));
        let new_scale_x =
            (aspect_ratio * cos * prev_scale_x + sin * prev_scale_y) / (aspect_ratio * cos + sin);
        let new_scale_y = aspect_ratio * (sin * target_scale_x + cos * target_scale_y)
            / ((cos + aspect_ratio * sin) * (aspect_ratio * cos + sin));
        self.set_scale_x(gc_context, new_scale_x);
        self.set_scale_y(gc_context, new_scale_y);
    }
    /// The opacity of this display object.
    /// 1 is fully opaque.
    /// Returned by the `_alpha`/`alpha` ActionScript properties.
    fn alpha(&self) -> f64;

    /// Sets the opacity of this display object.
    /// 1 is fully opaque.
    /// Set by the `_alpha`/`alpha` ActionScript properties.
    fn set_alpha(&self, gc_context: MutationContext<'gc, '_>, value: f64);
    fn name(&self) -> Ref<str>;
    fn set_name(&mut self, context: MutationContext<'gc, '_>, name: &str);

    /// Returns the dot-syntax path to this display object, e.g. `_level0.foo.clip`
    fn path(&self) -> String {
        if let Some(parent) = self.parent() {
            let mut path = parent.path();
            path.push_str(".");
            path.push_str(&*self.name());
            path
        } else {
            format!("_level{}", self.depth())
        }
    }

    /// Returns the Flash 4 slash-syntax path to this display object, e.g. `/foo/clip`.
    /// Returned by the `_target` property in AVM1.
    fn slash_path(&self) -> String {
        if let Some(parent) = self.parent() {
            let mut path = parent.slash_path();
            path.push_str("/");
            path.push_str(&*self.name());
            path
        } else {
            // The stage/root levels do not append their name in slash syntax.
            "".to_string()
        }
    }

    fn clip_depth(&self) -> Depth;
    fn set_clip_depth(&mut self, context: MutationContext<'gc, '_>, depth: Depth);
    fn parent(&self) -> Option<DisplayObject<'gc>>;
    fn set_parent(&mut self, context: MutationContext<'gc, '_>, parent: Option<DisplayObject<'gc>>);
    fn first_child(&self) -> Option<DisplayObject<'gc>>;
    fn set_first_child(
        &mut self,
        context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    );
    fn prev_sibling(&self) -> Option<DisplayObject<'gc>>;
    fn set_prev_sibling(
        &mut self,
        context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    );
    fn next_sibling(&self) -> Option<DisplayObject<'gc>>;
    fn set_next_sibling(
        &mut self,
        context: MutationContext<'gc, '_>,
        node: Option<DisplayObject<'gc>>,
    );
    /// Iterates over the children of this display object in execution order.
    /// This is different than render order.
    fn children(&self) -> ChildIter<'gc> {
        ChildIter {
            cur_child: self.first_child(),
        }
    }
    /// Get a child display object by instance name.
    fn get_child_by_name(&self, name: &str) -> Option<DisplayObject<'gc>> {
        // TODO: Make a HashMap from name -> child?
        self.children().find(|child| &*child.name() == name)
    }

    /// Get another level by level name.
    ///
    /// Since levels don't have instance names, this function instead parses
    /// their ID and uses that to retrieve the level.
    fn get_level_by_path(
        &self,
        name: &str,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Option<DisplayObject<'gc>> {
        if name.get(0..min(name.len(), 6)) == Some("_level") {
            if let Some(level_id) = name.get(6..).and_then(|v| v.parse::<u32>().ok()) {
                return context.levels.get(&level_id).copied();
            }
        }

        None
    }
    fn removed(&self) -> bool;
    fn set_removed(&mut self, context: MutationContext<'gc, '_>, value: bool);

    /// Whether this display object is visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    fn visible(&self) -> bool;

    /// Sets whether this display object will be visible.
    /// Invisible objects are not rendered, but otherwise continue to exist normally.
    /// Returned by the `_visible`/`visible` ActionScript properties.
    fn set_visible(&mut self, context: MutationContext<'gc, '_>, value: bool);

    /// Whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    fn transformed_by_script(&self) -> bool;

    /// Sets whether this display object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    fn set_transformed_by_script(&self, context: MutationContext<'gc, '_>, value: bool);

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed by its parent, and
    /// so forth.
    fn propagate_button_event(
        &self,
        context: &mut UpdateContext<'_, 'gc, '_>,
        event: ButtonEvent,
    ) -> ButtonEventResult {
        for child in self.children() {
            if child.propagate_button_event(context, event) == ButtonEventResult::Handled {
                return ButtonEventResult::Handled;
            }
        }
        ButtonEventResult::NotHandled
    }

    /// Executes and propagates the given clip event.
    /// Events execute inside-out; the deepest child will react first, followed by its parent, and
    /// so forth.
    fn propagate_clip_event(&self, context: &mut UpdateContext<'_, 'gc, '_>, event: ClipEvent) {
        for child in self.children() {
            child.propagate_clip_event(context, event);
        }
    }

    fn run_frame(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) {}
    fn render(&self, _context: &mut RenderContext<'_, 'gc>) {}
    fn unload(&mut self, context: &mut UpdateContext<'_, 'gc, '_>) {
        self.set_removed(context.gc_context, true);
    }
    fn as_button(&self) -> Option<Button<'gc>> {
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
    fn apply_place_object(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        place_object: &swf::PlaceObject,
    ) {
        // PlaceObject tags only apply if this onject has not been dynamically moved by AS code.
        if !self.transformed_by_script() {
            if let Some(matrix) = &place_object.matrix {
                self.set_matrix(gc_context, &matrix.clone().into());
            }
            if let Some(color_transform) = &place_object.color_transform {
                self.set_color_transform(gc_context, &color_transform.clone().into());
            }
            if let Some(name) = &place_object.name {
                self.set_name(gc_context, name);
            }
            if let Some(clip_depth) = place_object.clip_depth {
                self.set_clip_depth(gc_context, clip_depth.into());
            }
            if let Some(ratio) = place_object.ratio {
                if let Some(mut morph_shape) = self.as_morph_shape() {
                    morph_shape.set_ratio(gc_context, ratio);
                }
            }
            // Clip events only apply to movie clips.
            if let Some(clip) = self.as_movie_clip() {
                // Convert from `swf::ClipAction` to Ruffle's `ClipAction`.
                use crate::display_object::movie_clip::ClipAction;
                clip.set_clip_actions(
                    gc_context,
                    place_object
                        .clip_actions
                        .iter()
                        .cloned()
                        .map(|a| ClipAction::from_action_and_movie(a, clip.movie().unwrap()))
                        .collect(),
                );
            }
            // TODO: Others will go here eventually.
        }
    }

    fn copy_display_properties_from(
        &mut self,
        gc_context: MutationContext<'gc, '_>,
        other: DisplayObject<'gc>,
    ) {
        self.set_matrix(gc_context, &*other.matrix());
        self.set_color_transform(gc_context, &*other.color_transform());
        self.set_clip_depth(gc_context, other.clip_depth());
        self.set_name(gc_context, &*other.name());
        if let (Some(mut me), Some(other)) = (self.as_morph_shape(), other.as_morph_shape()) {
            me.set_ratio(gc_context, other.ratio());
        }
        // onEnterFrame actions only apply to movie clips.
        if let (Some(me), Some(other)) = (self.as_movie_clip(), other.as_movie_clip()) {
            me.set_clip_actions(gc_context, other.clip_actions().iter().cloned().collect());
        }
        // TODO: More in here eventually.
    }

    fn object(&self) -> Value<'gc> {
        Value::Undefined // todo: impl for every type and delete this fallback
    }

    /// Tests if a given stage position point intersects with the world bounds of this object.
    fn hit_test(&self, _pos: (Twips, Twips)) -> bool {
        false
    }

    fn mouse_pick(
        &self,
        _self_node: DisplayObject<'gc>,
        _pos: (Twips, Twips),
    ) -> Option<DisplayObject<'gc>> {
        None
    }

    fn post_instantiation(
        &mut self,
        _gc_context: MutationContext<'gc, '_>,
        _display_object: DisplayObject<'gc>,
        _proto: Object<'gc>,
    ) {
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

    /// Obtain the top-most parent of the display tree hierarchy.
    ///
    /// This function can panic in the rare case that a top-level display
    /// object has not been post-instantiated, or that a top-level display
    /// object does not implement `object`.
    fn root(&self) -> DisplayObject<'gc> {
        let mut parent = self.parent();

        while let Some(p) = parent {
            let grandparent = p.parent();

            if grandparent.is_none() {
                break;
            }

            parent = grandparent;
        }

        parent
            .or_else(|| {
                self.object()
                    .as_object()
                    .ok()
                    .and_then(|o| o.as_display_object())
            })
            .expect("All objects must have root")
    }
}

pub enum DisplayObjectPtr {}

// To use this macro: `use crate::impl_display_object;` or `use crate::prelude::*;`
#[macro_export]
macro_rules! impl_display_object {
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
        fn set_place_frame(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, frame: u16) {
            self.0.write(context).$field.set_place_frame(context, frame)
        }
        fn transform(&self) -> std::cell::Ref<crate::transform::Transform> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.transform())
        }
        fn matrix(&self) -> std::cell::Ref<crate::matrix::Matrix> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.matrix())
        }
        fn matrix_mut(&mut self,
            context: gc_arena::MutationContext<'gc, '_>) -> std::cell::RefMut<crate::matrix::Matrix> {
            std::cell::RefMut::map(self.0.write(context), |o| o.$field.matrix_mut(context))
        }
        fn set_matrix(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, matrix: &crate::matrix::Matrix) {
            self.0.write(context).$field.set_matrix(context, matrix)
        }
        fn color_transform(&self) -> std::cell::Ref<crate::color_transform::ColorTransform> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.color_transform())
        }
        fn color_transform_mut(&self, context: gc_arena::MutationContext<'gc, '_>) -> std::cell::RefMut<crate::color_transform::ColorTransform> {
            std::cell::RefMut::map(self.0.write(context), |o| o.$field.color_transform_mut())
        }
        fn set_color_transform(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, color_transform: &crate::color_transform::ColorTransform) {
            self.0.write(context).$field.set_color_transform(context, color_transform)
        }
        fn x(&self) -> f64 {
            self.0.read().$field.x()
        }
        fn set_x(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_x(value)
        }
        fn y(&self) -> f64 {
            self.0.read().$field.y()
        }
        fn set_y(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_y(value)
        }
        fn rotation(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>) -> f64 {
            self.0.write(gc_context).$field.rotation()
        }
        fn set_rotation(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>, radians: f64) {
            self.0.write(gc_context).$field.set_rotation(radians)
        }
        fn scale_x(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>) -> f64 {
            self.0.write(gc_context).$field.scale_x()
        }
        fn set_scale_x(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_scale_x(value)
        }
        fn scale_y(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>) -> f64 {
            self.0.write(gc_context).$field.scale_y()
        }
        fn set_scale_y(&mut self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_scale_y(value)
        }
        fn alpha(&self) -> f64 {
            self.0.read().$field.alpha()
        }
        fn set_alpha(&self, gc_context: gc_arena::MutationContext<'gc, '_>, value: f64) {
            self.0.write(gc_context).$field.set_alpha(value)
        }
        fn name(&self) -> std::cell::Ref<str> {
            std::cell::Ref::map(self.0.read(), |o| o.$field.name())
        }
        fn set_name(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, name: &str) {
            self.0.write(context).$field.set_name(context, name)
        }
        fn clip_depth(&self) -> crate::prelude::Depth {
            self.0.read().$field.clip_depth()
        }
        fn set_clip_depth(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, depth: crate::prelude::Depth) {
            self.0.write(context).$field.set_clip_depth(context, depth)
        }
        fn parent(&self) -> Option<crate::display_object::DisplayObject<'gc>> {
            self.0.read().$field.parent()
        }
        fn set_parent(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, parent: Option<crate::display_object::DisplayObject<'gc>>) {
            self.0.write(context).$field.set_parent(context, parent)
        }
        fn first_child(&self) -> Option<DisplayObject<'gc>> {
            self.0.read().$field.first_child()
        }
        fn set_first_child(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, node: Option<DisplayObject<'gc>>) {
            self.0.write(context).$field.set_first_child(context, node);
        }
        fn prev_sibling(&self) -> Option<DisplayObject<'gc>> {
            self.0.read().$field.prev_sibling()
        }
        fn set_prev_sibling(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, node: Option<DisplayObject<'gc>>) {
            self.0.write(context).$field.set_prev_sibling(context, node);
        }
        fn next_sibling(&self) -> Option<DisplayObject<'gc>> {
            self.0.read().$field.next_sibling()
        }
        fn set_next_sibling(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, node: Option<DisplayObject<'gc>>) {
            self.0.write(context).$field.set_next_sibling(context, node);
        }
        fn removed(&self) -> bool {
            self.0.read().$field.removed()
        }
        fn set_removed(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_removed(value)
        }
        fn visible(&self) -> bool {
            self.0.read().$field.visible()
        }
        fn set_visible(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_visible(value);
        }
        fn transformed_by_script(&self) -> bool {
            self.0.read().$field.transformed_by_script()
        }
        fn set_transformed_by_script(&self, context: gc_arena::MutationContext<'gc, '_>, value: bool) {
            self.0.write(context).$field.set_transformed_by_script(value)
        }
        fn swf_version(&self) -> u8 {
            self.0.read().$field.swf_version()
        }
        fn instantiate(&self, gc_context: gc_arena::MutationContext<'gc, '_>) -> crate::display_object::DisplayObject<'gc> {
            Self(gc_arena::GcCell::allocate(gc_context, self.0.read().clone())).into()
        }
        fn as_ptr(&self) -> *const crate::display_object::DisplayObjectPtr {
            self.0.as_ptr() as *const crate::display_object::DisplayObjectPtr
        }
    };
}

/// Renders the children of a display object, taking masking into account.
// TODO(Herschel): Move this into an IDisplayObject/IDisplayObjectContainer trait when
// we figure out inheritance
pub fn render_children<'gc>(
    context: &mut RenderContext<'_, 'gc>,
    children: &std::collections::BTreeMap<Depth, DisplayObject<'gc>>,
) {
    let mut clip_depth = 0;
    let mut clip_depth_stack = vec![];
    for (&depth, &child) in children {
        // Check if we need to pop off a mask.
        // This must be a while loop because multiple masks can be popped
        // at the same dpeth.
        while clip_depth > 0 && depth >= clip_depth {
            context.renderer.pop_mask();
            clip_depth = clip_depth_stack.pop().unwrap();
        }
        if child.clip_depth() > 0 && child.allow_as_mask() {
            // Push and render the mask.
            clip_depth_stack.push(clip_depth);
            clip_depth = child.clip_depth();
            context.renderer.push_mask();
            child.render(context);
            context.renderer.activate_mask();
        } else if child.visible() {
            // Normal child.
            child.render(context);
        }
    }

    while !clip_depth_stack.is_empty() {
        context.renderer.pop_mask();
        clip_depth_stack.pop();
    }
}

impl<'gc> DisplayObject<'gc> {
    pub fn ptr_eq(a: DisplayObject<'gc>, b: DisplayObject<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
}

/// Bit flags used by `DisplayObject`.
#[derive(Collect, EnumSetType, Debug)]
#[collect(no_drop)]
enum DisplayObjectFlags {
    /// Whether this object has been removed from the display list.
    /// Necessary in AVM1 to throw away queued actions from removed movie clips.
    Removed,

    /// If this object is visible (`_visible` property).
    Visible,

    /// Whether the `_xscale`, `_yscale` and `_rotation` of the object have been calculated and cached.
    ScaleRotationCached,

    /// Whether this object has been transformed by ActionScript.
    /// When this flag is set, changes from SWF `PlaceObject` tags are ignored.
    TransformedByScript,
}

pub struct ChildIter<'gc> {
    cur_child: Option<DisplayObject<'gc>>,
}

impl<'gc> Iterator for ChildIter<'gc> {
    type Item = DisplayObject<'gc>;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur_child;
        self.cur_child = self
            .cur_child
            .and_then(|display_cell| display_cell.next_sibling());
        cur
    }
}
