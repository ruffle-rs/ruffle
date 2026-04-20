use crate::avm2::activation::Activation;
use crate::avm2::error::Error;
use crate::avm2::globals::vector::concat_helper;
use crate::avm2::value::Value;

#[rustfmt::skip]
pub use crate::avm2::globals::vector::{
    call_handler,
    instance_init as vector_int_initializer,

    get_fixed,
    set_fixed,
    get_length,
    set_length,

    every,
    filter,
    for_each,
    index_of,
    insert_at,
    join,
    last_index_of,
    map,
    pop,
    push,
    remove_at,
    reverse,
    shift,
    slice,
    _some,
    sort,
    splice,
    unshift,
};

pub use crate::avm2::object::vector_allocator as vector_int_allocator;

/// `Vector.concat` impl
pub fn concat<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let vector_class = activation.avm2().class_defs().int_vector;

    concat_helper(activation, vector_class, this, args)
}
