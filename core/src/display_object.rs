use crate::avm1::{Object, Value};
use crate::context::{RenderContext, UpdateContext};
use crate::player::NEWEST_PLAYER_VERSION;
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, MutationContext};
use ruffle_macros::enum_trait_object;
use std::cell::{Ref, RefMut};
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
#[collect(no_drop)]
pub struct DisplayObjectBase<'gc> {
    parent: Option<DisplayObject<'gc>>,
    place_frame: u16,
    depth: Depth,
    transform: Transform,
    name: String,
    clip_depth: Depth,

    /// The first child of this display object in order of execution.
    /// This is differen than render order.
    first_child: Option<DisplayObject<'gc>>,

    /// The previous sibling of this display object in order of execution.
    prev_sibling: Option<DisplayObject<'gc>>,

    /// The next sibling of this display object in order of execution.
    next_sibling: Option<DisplayObject<'gc>>,

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

#[allow(dead_code)]
impl<'gc> DisplayObjectBase<'gc> {
    fn id(&self) -> CharacterId {
        0
    }
    fn depth(&self) -> Depth {
        self.depth
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
    }
    fn color_transform(&self) -> &ColorTransform {
        &self.transform.color_transform
    }
    fn set_color_transform(
        &mut self,
        _context: MutationContext<'gc, '_>,
        color_transform: &ColorTransform,
    ) {
        self.transform.color_transform = *color_transform;
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn set_name(&mut self, _context: MutationContext<'gc, '_>, name: &str) {
        self.name = name.to_string();
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
        self.removed
    }
    fn set_removed(&mut self, _context: MutationContext<'gc, '_>, removed: bool) {
        self.removed = removed;
    }
    fn swf_version(&self) -> u8 {
        self.parent
            .map(|p| p.swf_version())
            .unwrap_or(NEWEST_PLAYER_VERSION)
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
pub trait TDisplayObject<'gc>: 'gc + Collect + Debug {
    fn id(&self) -> CharacterId;
    fn depth(&self) -> Depth;
    fn local_bounds(&self) -> BoundingBox {
        BoundingBox::default()
    }

    fn world_bounds(&self) -> BoundingBox {
        BoundingBox::default()
    }
    fn place_frame(&self) -> u16;
    fn set_place_frame(&mut self, context: MutationContext<'gc, '_>, frame: u16);

    fn transform(&self) -> Ref<Transform>;
    fn matrix(&self) -> Ref<Matrix>;
    fn matrix_mut(&mut self, context: MutationContext<'gc, '_>) -> RefMut<Matrix>;
    fn set_matrix(&mut self, context: MutationContext<'gc, '_>, matrix: &Matrix);
    fn color_transform(&self) -> Ref<ColorTransform>;
    fn set_color_transform(
        &mut self,
        context: MutationContext<'gc, '_>,
        color_transform: &ColorTransform,
    );
    fn name(&self) -> Ref<str>;
    fn set_name(&mut self, context: MutationContext<'gc, '_>, name: &str);
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
    fn removed(&self) -> bool;
    fn set_removed(&mut self, context: MutationContext<'gc, '_>, removed: bool);
    fn run_frame(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) {}
    fn render(&self, _context: &mut RenderContext<'_, 'gc>) {}

    fn as_button(&self) -> Option<Button<'gc>> {
        None
    }
    fn as_movie_clip(&self) -> Option<MovieClip<'gc>> {
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
            self.set_clip_depth(gc_context, clip_depth);
        }
        if let Some(ratio) = place_object.ratio {
            if let Some(mut morph_shape) = self.as_morph_shape() {
                morph_shape.set_ratio(gc_context, ratio);
            }
        }
        // TODO: Others will go here eventually.
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
        // TODO: More in here eventually.
    }

    fn object(&self) -> Value<'gc> {
        Value::Undefined // todo: impl for every type and delete this fallback
    }

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

    fn instantiate(&self, gc_context: MutationContext<'gc, '_>) -> DisplayObject<'gc>;
    fn as_ptr(&self) -> *const DisplayObjectPtr;
}

pub enum DisplayObjectPtr {}

// To use this macro: `use crate::impl_display_object;` or `use crate::prelude::*;`
#[macro_export]
macro_rules! impl_display_object {
    ($field:ident) => {
        fn depth(&self) -> crate::prelude::Depth {
            self.0.read().$field.depth()
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
        fn set_color_transform(&mut self,
            context: gc_arena::MutationContext<'gc, '_>, color_transform: &crate::color_transform::ColorTransform) {
            self.0.write(context).$field.set_color_transform(context, color_transform)
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
            self.0.write(context).$field.set_removed(context, value)
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

impl<'gc> DisplayObject<'gc> {
    pub fn ptr_eq(a: DisplayObject<'gc>, b: DisplayObject<'gc>) -> bool {
        a.as_ptr() == b.as_ptr()
    }
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
