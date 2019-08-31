use crate::player::{RenderContext, UpdateContext};
use crate::prelude::*;
use crate::transform::Transform;
use gc_arena::{Collect, GcCell, MutationContext};
use std::fmt::Debug;
use crate::avm1::Value;

#[derive(Clone, Collect, Debug)]
#[collect(empty_drop)]
pub struct DisplayObjectBase<'gc> {
    parent: Option<DisplayNode<'gc>>,
    depth: Depth,
    transform: Transform,
    name: String,
    clip_depth: Depth,
}

impl<'gc> Default for DisplayObjectBase<'gc> {
    fn default() -> Self {
        Self {
            parent: Default::default(),
            depth: Default::default(),
            transform: Default::default(),
            name: Default::default(),
            clip_depth: Default::default(),
        }
    }
}

impl<'gc> DisplayObject<'gc> for DisplayObjectBase<'gc> {
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
    fn box_clone(&self) -> Box<dyn DisplayObject<'gc>> {
        Box::new(self.clone())
    }
}

pub trait DisplayObject<'gc>: 'gc + Collect + Debug {
    fn local_bounds(&self) -> BoundingBox {
        BoundingBox::default()
    }

    fn world_bounds(&self) -> BoundingBox {
        BoundingBox::default()
    }

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

    fn preload(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) {}
    fn run_frame(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) {}
    fn run_post_frame(&mut self, _context: &mut UpdateContext<'_, 'gc, '_>) {}
    fn render(&self, _context: &mut RenderContext<'_, 'gc>) {}

    fn as_button(&self) -> Option<&crate::button::Button<'gc>> {
        None
    }
    fn as_button_mut(&mut self) -> Option<&mut crate::button::Button<'gc>> {
        None
    }
    fn as_movie_clip(&self) -> Option<&crate::movie_clip::MovieClip<'gc>> {
        None
    }
    fn as_movie_clip_mut(&mut self) -> Option<&mut crate::movie_clip::MovieClip<'gc>> {
        None
    }
    fn as_morph_shape(&self) -> Option<&crate::morph_shape::MorphShape<'gc>> {
        None
    }
    fn as_morph_shape_mut(&mut self) -> Option<&mut crate::morph_shape::MorphShape<'gc>> {
        None
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
}

impl<'gc> Clone for Box<dyn DisplayObject<'gc>> {
    fn clone(&self) -> Box<dyn DisplayObject<'gc>> {
        self.box_clone()
    }
}

macro_rules! impl_display_object {
    ($field:ident) => {
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
        fn box_clone(&self) -> Box<dyn crate::display_object::DisplayObject<'gc>> {
            Box::new(self.clone())
        }
    };
}

/// `DisplayNode` is the garbage-collected pointer between display objects.
/// TODO(Herschel): The extra Box here is necessary to hold the trait object inside a GC pointer,
/// but this is an extra allocation... Can we avoid this, maybe with a DST?
pub type DisplayNode<'gc> = GcCell<'gc, Box<dyn DisplayObject<'gc>>>;
