//! Property map

use crate::avm2::names::QName;
use std::collections::HashMap;

/// Type which represents named properties on an object.
pub type PropertyMap<'gc, V> = HashMap<QName<'gc>, V>;
