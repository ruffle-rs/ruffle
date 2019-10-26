use crate::avm1::Value;
use crate::player::{RenderContext, UpdateContext, NEWEST_PLAYER_VERSION};
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt::Debug;

mod bitmap;
mod button;
mod edit_text;
mod graphic;
mod morph_shape;
mod movie_clip;
mod text;

pub use bitmap::Bitmap;
pub use button::Button;
pub use edit_text::EditText;
pub use graphic::Graphic;
pub use morph_shape::{MorphShape, MorphShapeStatic};
pub use movie_clip::MovieClip;
pub use text::Text;

#[derive(Clone, Collect, Debug)]
#[collect(empty_drop)]
pub struct DisplayObjectBase<'gc> {
    parent: Option<DisplayNode<'gc>>,
    place_frame: u16,
    depth: Depth,
    transform: Transform,
    name: String,
    clip_depth: Depth,

    /// The first child of this display object in order of execution.
    /// This is differen than render order.
    first_child: Option<DisplayNode<'gc>>,

    /// The previous sibling of this display object in order of execution.
    prev_sibling: Option<DisplayNode<'gc>>,

    /// The next sibling of this display object in order of execution.
    next_sibling: Option<DisplayNode<'gc>>,

    /// Whether this child has been removed from the display list.
    /// Necessary in AVM1 to throw away queued actions from removed movie clips.
    removed: bool,
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
            first_child: None,
            prev_sibling: None,
            next_sibling: None,
            removed: false,
        }
    }
}

impl<'gc> DisplayObject<'gc> for DisplayObjectBase<'gc> {
    fn id(&self) -> CharacterId {
        0
    }
    fn depth(&self) -> Depth {
        self.depth
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
    }
    fn color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }
    fn set_color_transform(&mut self, color_transform: &ColorTransform) {
        self.transform.color_transform = *color_transform;
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    fn clip_depth(&self) -> Depth {
        self.clip_depth
    }
    fn set_clip_depth(&mut self, depth: Depth) {
        self.clip_depth = depth;
    }
    fn parent(&self) -> Option<DisplayNode<'gc>> {
        self.parent
    }
    fn set_parent(&mut self, parent: Option<DisplayNode<'gc>>) {
        self.parent = parent;
    }
    fn first_child(&self) -> Option<DisplayNode<'gc>> {
        self.first_child
    }
    fn set_first_child(&mut self, node: Option<DisplayNode<'gc>>) {
        self.first_child = node;
    }
    fn prev_sibling(&self) -> Option<DisplayNode<'gc>> {
        self.prev_sibling
    }
    fn set_prev_sibling(&mut self, node: Option<DisplayNode<'gc>>) {
        self.prev_sibling = node;
    }
    fn next_sibling(&self) -> Option<DisplayNode<'gc>> {
        self.next_sibling
    }
    fn set_next_sibling(&mut self, node: Option<DisplayNode<'gc>>) {
        self.next_sibling = node;
    }
    fn removed(&self) -> bool {
        self.removed
    }
    fn set_removed(&mut self, removed: bool) {
        self.removed = removed;
    }
    fn box_clone(&self) -> Box<dyn DisplayObject<'gc>> {
        Box::new(self.clone())
    }
}

pub trait DisplayObject<'gc>: 'gc + Collect + Debug {
    fn id(&self) -> CharacterId;
    fn depth(&self) -> Depth;
    fn local_bounds(&self) -> BoundingBox {
        BoundingBox::default()
    }

    fn world_bounds(&self) -> BoundingBox {
        BoundingBox::default()
    }
    fn place_frame(&self) -> u16;
    fn set_place_frame(&mut self, frame: u16);

    fn transform(&self) -> &Transform;
    fn matrix(&self) -> &Matrix;
    fn matrix_mut(&mut self) -> &mut Matrix;
    fn set_matrix(&mut self, matrix: &Matrix);
    fn color_transform(&self) -> &ColorTransform;
    fn set_color_transform(&mut self, color_transform: &ColorTransform);
    fn name(&self) -> &str;
    fn set_name(&mut self, name: &str);
    fn clip_depth(&self) -> Depth;
    fn set_clip_depth(&mut self, depth: Depth);
    fn parent(&self) -> Option<DisplayNode<'gc>>;
    fn set_parent(&mut self, parent: Option<DisplayNode<'gc>>);
    fn first_child(&self) -> Option<DisplayNode<'gc>>;
    fn set_first_child(&mut self, node: Option<DisplayNode<'gc>>);
    fn prev_sibling(&self) -> Option<DisplayNode<'gc>>;
    fn set_prev_sibling(&mut self, node: Option<DisplayNode<'gc>>);
    fn next_sibling(&self) -> Option<DisplayNode<'gc>>;
    fn set_next_sibling(&mut self, node: Option<DisplayNode<'gc>>);
    /// Iterates over the children of this display object in execution order.
    /// This is different than render order.
    fn children(&self) -> ChildIter<'gc> {
        ChildIter {
            cur_child: self.first_child(),
        }
    }
    fn removed(&self) -> bool;
    fn set_removed(&mut self, removed: bool);
    fn run_frame(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) {}
    fn render(&self, _context: &mut RenderContext<'_, 'gc>) {}

    fn as_button(&self) -> Option<&Button<'gc>> {
        None
    }
    fn as_button_mut(&mut self) -> Option<&mut Button<'gc>> {
        None
    }
    fn as_movie_clip(&self) -> Option<&MovieClip<'gc>> {
        None
    }
    fn as_movie_clip_mut(&mut self) -> Option<&mut MovieClip<'gc>> {
        None
    }
    fn as_morph_shape(&self) -> Option<&MorphShape<'gc>> {
        None
    }
    fn as_morph_shape_mut(&mut self) -> Option<&mut MorphShape<'gc>> {
        None
    }
    fn apply_place_object(&mut self, place_object: &swf::PlaceObject) {
        if let Some(matrix) = &place_object.matrix {
            self.set_matrix(&matrix.clone().into());
        }
        if let Some(color_transform) = &place_object.color_transform {
            self.set_color_transform(&color_transform.clone().into());
        }
        if let Some(name) = &place_object.name {
            self.set_name(name);
        }
        if let Some(clip_depth) = place_object.clip_depth {
            self.set_clip_depth(clip_depth);
        }
        if let Some(ratio) = place_object.ratio {
            if let Some(morph_shape) = self.as_morph_shape_mut() {
                morph_shape.set_ratio(ratio);
            }
        }
        // TODO: Others will go here eventually.
    }

    fn copy_display_properties_from(&mut self, other: DisplayNode<'gc>) {
        let other = other.read();
        self.set_matrix(other.matrix());
        self.set_color_transform(other.color_transform());
        self.set_clip_depth(other.clip_depth());
        self.set_name(other.name());
        if let (Some(me), Some(other)) = (self.as_morph_shape_mut(), other.as_morph_shape()) {
            me.set_ratio(other.ratio());
        }
        // TODO: More in here eventually.
    }

    fn box_clone(&self) -> Box<dyn DisplayObject<'gc>>;

    fn object(&self) -> Value<'gc> {
        Value::Undefined // todo: impl for every type and delete this fallback
    }

    fn hit_test(&self, _: (Twips, Twips)) -> bool {
        false
    }

    fn mouse_pick(
        &self,
        _self_node: DisplayNode<'gc>,
        _: (Twips, Twips),
    ) -> Option<DisplayNode<'gc>> {
        None
    }

    fn post_instantiation(
        &mut self,
        _gc_context: MutationContext<'gc, '_>,
        _display_object: DisplayNode<'gc>,
    ) {
    }

    /// Return the version of the SWF that created this movie clip.
    fn swf_version(&self) -> u8 {
        self.parent()
            .map(|p| p.read().swf_version())
            .unwrap_or(NEWEST_PLAYER_VERSION)
    }
}

impl<'gc> Clone for Box<dyn DisplayObject<'gc>> {
    fn clone(&self) -> Box<dyn DisplayObject<'gc>> {
        self.box_clone()
    }
}

// To use this macro: `use crate::impl_display_object;` or `use crate::prelude::*;`
#[macro_export]
macro_rules! impl_display_object {
    ($field:ident) => {
        fn depth(&self) -> crate::prelude::Depth {
            self.$field.depth()
        }
        fn place_frame(&self) -> u16 {
            self.$field.place_frame()
        }
        fn set_place_frame(&mut self, frame: u16) {
            self.$field.set_place_frame(frame)
        }
        fn transform(&self) -> &crate::transform::Transform {
            self.$field.transform()
        }
        fn matrix(&self) -> &crate::matrix::Matrix {
            self.$field.matrix()
        }
        fn matrix_mut(&mut self) -> &mut crate::matrix::Matrix {
            self.$field.matrix_mut()
        }
        fn set_matrix(&mut self, matrix: &crate::matrix::Matrix) {
            self.$field.set_matrix(matrix)
        }
        fn color_transform(&self) -> &crate::color_transform::ColorTransform {
            self.$field.color_transform()
        }
        fn set_color_transform(&mut self, color_transform: &crate::color_transform::ColorTransform) {
            self.$field.set_color_transform(color_transform)
        }
        fn name(&self) -> &str {
            self.$field.name()
        }
        fn set_name(&mut self, name: &str) {
            self.$field.set_name(name)
        }
        fn clip_depth(&self) -> crate::prelude::Depth {
            self.$field.clip_depth()
        }
        fn set_clip_depth(&mut self, depth: crate::prelude::Depth) {
            self.$field.set_clip_depth(depth)
        }
        fn parent(&self) -> Option<crate::display_object::DisplayNode<'gc>> {
            self.$field.parent()
        }
        fn set_parent(&mut self, parent: Option<crate::display_object::DisplayNode<'gc>>) {
            self.$field.set_parent(parent)
        }
        fn first_child(&self) -> Option<DisplayNode<'gc>> {
            self.$field.first_child()
        }
        fn set_first_child(&mut self, node: Option<DisplayNode<'gc>>) {
            self.$field.set_first_child(node);
        }
        fn prev_sibling(&self) -> Option<DisplayNode<'gc>> {
            self.$field.prev_sibling()
        }
        fn set_prev_sibling(&mut self, node: Option<DisplayNode<'gc>>) {
            self.$field.set_prev_sibling(node);
        }
        fn next_sibling(&self) -> Option<DisplayNode<'gc>> {
            self.$field.next_sibling()
        }
        fn set_next_sibling(&mut self, node: Option<DisplayNode<'gc>>) {
            self.$field.set_next_sibling(node);
        }
        fn removed(&self) -> bool {
            self.$field.removed()
        }
        fn set_removed(&mut self, value: bool) {
            self.$field.set_removed(value)
        }
        fn box_clone(&self) -> Box<dyn crate::display_object::DisplayObject<'gc>> {
            Box::new(self.clone())
        }
        fn swf_version(&self) -> u8 {
            self.$field.swf_version()
        }
    };
}

/// Renders the children of a display object, taking masking into account.
// TODO(Herschel): Move this into an IDisplayObject/IDisplayObjectContainer trait when
// we figure out inheritance
pub fn render_children<'gc>(
    context: &mut RenderContext<'_, 'gc>,
    children: &std::collections::BTreeMap<Depth, DisplayNode<'gc>>,
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
        let child = child.read();
        if child.clip_depth() > 0 {
            // Push and render the mask.
            clip_depth_stack.push(clip_depth);
            clip_depth = child.clip_depth();
            context.renderer.push_mask();
            child.render(context);
            context.renderer.activate_mask();
        } else {
            // Normal child.
            child.render(context);
        }
    }

    while !clip_depth_stack.is_empty() {
        context.renderer.pop_mask();
        clip_depth_stack.pop();
    }
}

/// `DisplayNode` is the garbage-collected pointer between display objects.
/// TODO(Herschel): The extra Box here is necessary to hold the trait object inside a GC pointer,
/// but this is an extra allocation... Can we avoid this, maybe with a DST?
pub type DisplayNode<'gc> = GcCell<'gc, Box<dyn DisplayObject<'gc>>>;

pub struct ChildIter<'gc> {
    cur_child: Option<DisplayNode<'gc>>,
}

impl<'gc> Iterator for ChildIter<'gc> {
    type Item = DisplayNode<'gc>;
    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur_child;
        self.cur_child = self
            .cur_child
            .and_then(|display_cell| display_cell.read().next_sibling());
        cur
    }
}
