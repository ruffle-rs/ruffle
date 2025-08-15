#[rustfmt::skip]
pub use crate::avm2::globals::vector::{
    call_handler,
    instance_init as vector_int_initializer,

    get_fixed,
    set_fixed,
    get_length,
    set_length,

    concat,
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
