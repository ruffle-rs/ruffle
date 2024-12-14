//! Declarative macro for defining AVM1 properties.

use std::borrow::Cow;

use crate::avm1::function::{Executable, FunctionObject, NativeFunction};
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::string::{AvmAtom, StringContext};

/// Defines a list of properties on a [`ScriptObject`].
#[inline(never)]
pub fn define_properties_on<'gc>(
    decls: &[Declaration],
    context: &mut StringContext<'gc>,
    this: ScriptObject<'gc>,
    fn_proto: Object<'gc>,
) {
    for decl in decls {
        decl.define_on(context, this, fn_proto);
    }
}

/// The declaration of a property, method, or simple field, that
/// can be defined on a [`ScriptObject`].
#[derive(Copy, Clone)]
pub struct Declaration {
    pub name: &'static str,
    pub kind: DeclKind,
    pub attributes: Attribute,
}

/// All the possible types of a [`Declaration`].
#[derive(Copy, Clone)]
pub enum DeclKind {
    /// Declares a property with a getter and an optional setter.
    Property {
        getter: NativeFunction,
        setter: Option<NativeFunction>,
    },
    /// Declares a native host function.
    ///
    /// This is intended for use with defining host object prototypes. Notably,
    /// this creates a function object without an explicit `prototype`, which
    /// is only possible when defining host functions.
    Method(NativeFunction),
    /// Declares a native function with a `prototype`.
    /// Prefer using [`Self::Method`] when defining host functions.
    Function(NativeFunction),
    /// Declares a static string value.
    String(&'static str),
    /// Declares a static bool value.
    Bool(bool),
    /// Declares a static int value.
    Int(i32),
    /// Declares a static float value.
    Float(f64),
}

impl Declaration {
    #[inline(never)]
    /// Defines the field represented by this declaration on a [`ScriptObject`].
    /// Returns the value defined on the object, or `undefined` if this declaration
    /// defined a property.
    pub fn define_on<'gc>(
        &self,
        context: &mut StringContext<'gc>,
        this: ScriptObject<'gc>,
        fn_proto: Object<'gc>,
    ) -> Value<'gc> {
        let mc = context.gc_context;

        let mut intern_utf8 = |s: &'static str| -> AvmAtom<'gc> {
            match ruffle_wstr::from_utf8(s) {
                Cow::Borrowed(s) => context.intern_static(s),
                Cow::Owned(s) => context.intern_wstr(s),
            }
        };

        let name = intern_utf8(self.name);
        let value = match self.kind {
            DeclKind::Property { getter, setter } => {
                let getter =
                    FunctionObject::function(mc, Executable::Native(getter), fn_proto, fn_proto);
                let setter = setter.map(|setter| {
                    FunctionObject::function(mc, Executable::Native(setter), fn_proto, fn_proto)
                });
                this.add_property(mc, name.into(), getter, setter, self.attributes);
                return Value::Undefined;
            }
            DeclKind::Method(func) => {
                FunctionObject::bare_function(mc, Some(Executable::Native(func)), None, fn_proto)
                    .into()
            }
            DeclKind::Function(func) => {
                FunctionObject::function(mc, Executable::Native(func), fn_proto, fn_proto).into()
            }
            DeclKind::String(s) => intern_utf8(s).into(),
            DeclKind::Bool(b) => b.into(),
            DeclKind::Int(i) => i.into(),
            DeclKind::Float(f) => f.into(),
        };

        this.define_value(mc, name, value, self.attributes);
        value
    }
}

/// Declares a list of property [`Declaration`]s that can be later defined on [`ScriptObject`]s.
///
/// # Usage:
///
/// ```rust,ignore
/// const DECLS: &'static [Declaration] = declare_properties! {
///     "length" => property(get_length);
///     "filters" => property(get_filters, set_filters);
///     "to_string" => method(to_string);
///     "to_string2" => function(to_string);
///     "locale" => string("en-US");
///     "enabled" => bool(true);
///     "size" => int(123);
///     "scale" => float(0.85);
///     // all declarations can also specify attributes
///     "hidden" => string("shh!"; DONT_ENUM | DONT_DELETE | READ_ONLY);
/// };
/// ```
#[allow(unused_macro_rules)]
macro_rules! declare_properties {
    ( $($name:literal => $kind:ident($($args:tt)*);)* ) => {
        &[ $(
            declare_properties!(@__prop $kind($name, $($args)*))
        ),* ]
    };
    (@__prop $kind:ident($name:literal $(,$args:expr)*) ) => {
        crate::avm1::property_decl::Declaration {
            name: $name,
            kind: declare_properties!(@__kind $kind ($($args),*)),
            attributes: crate::avm1::property::Attribute::empty(),
        }
    };
    (@__prop $kind:ident($name:literal $(,$args:expr)*; $($attributes:ident)|*) ) => {
        crate::avm1::property_decl::Declaration {
            name: $name,
            kind: declare_properties!(@__kind $kind ($($args),*)),
            attributes: crate::avm1::property::Attribute::from_bits_truncate(
                0 $(| crate::avm1::property::Attribute::$attributes.bits())*
            ),
        }
    };
    (@__kind property($getter:expr)) => {
        crate::avm1::property_decl::DeclKind::Property {
            getter: $getter,
            setter: None,
        }
    };
    (@__kind property($getter:expr, $setter:expr)) => {
        crate::avm1::property_decl::DeclKind::Property {
            getter: $getter,
            setter: Some($setter),
        }
    };
    (@__kind method($method:expr)) => {
        crate::avm1::property_decl::DeclKind::Method($method)
    };
    (@__kind function($function:expr)) => {
        crate::avm1::property_decl::DeclKind::Function($function)
    };
    (@__kind string($string:expr)) => {
        crate::avm1::property_decl::DeclKind::String($string)
    };
    (@__kind bool($boolean:expr)) => {
        crate::avm1::property_decl::DeclKind::Bool($boolean)
    };
    (@__kind int($int:expr)) => {
        crate::avm1::property_decl::DeclKind::Int($int)
    };
    (@__kind float($float:expr)) => {
        crate::avm1::property_decl::DeclKind::Float($float)
    };
}
