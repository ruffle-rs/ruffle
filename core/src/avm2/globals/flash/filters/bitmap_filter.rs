//! `flash.filters.BitmapFilter`

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_2012;
use crate::avm2::object::{ClassObject, Object, scriptobject_allocator};
use crate::avm2::{Class, Error};

/// Implements `BitmapFilter`'s instance allocator.
///
/// Only the engine-recognized concrete filter subclasses can be constructed;
/// `BitmapFilter` itself and any other custom subclass throws.
pub fn bitmap_filter_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Object<'gc>, Error<'gc>> {
    let instance_class = class.inner_class_definition();

    if is_bitmap_filter_constructible(activation, instance_class) {
        scriptobject_allocator(class, activation)
    } else {
        let class_name = instance_class.name().local_name();
        Err(make_error_2012(activation, format!("{}$", class_name)))
    }
}

fn is_bitmap_filter_constructible<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: Class<'gc>,
) -> bool {
    let classes = activation.avm2().classes();
    let known_filters = [
        classes.bevelfilter,
        classes.blurfilter,
        classes.colormatrixfilter,
        classes.convolutionfilter,
        classes.displacementmapfilter,
        classes.dropshadowfilter,
        classes.glowfilter,
        classes.gradientbevelfilter,
        classes.gradientglowfilter,
        classes.shaderfilter,
    ];

    known_filters
        .iter()
        .any(|filter_class| class.has_class_in_chain(filter_class.inner_class_definition()))
}
