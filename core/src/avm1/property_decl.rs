//! Declarative macro for defining AVM1 properties.

use crate::avm1::function::{Executable, FunctionObject, NativeFunction};
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use gc_arena::MutationContext;

/// Defines a list of properties on a [`ScriptObject`].
#[inline(never)]
pub fn define_properties_on<'gc>(
    decls: &[Declaration],
    mc: MutationContext<'gc, '_>,
    this: ScriptObject<'gc>,
    fn_proto: Object<'gc>,
) {
    for decl in decls {
        decl.define_on(mc, this, fn_proto);
    }
}

/// The declaration of a property, method, or simple field, that
/// can be defined on a [`ScriptObject`].
#[derive(Copy, Clone)]
pub struct Declaration {
    pub name: &'static str,
    pub kind: DeclKind,
    // This should be an `Attribute`, but because of `const` shenanigans
    // we need to store the raw flags.
    // See the comment in the `declare_properties!` macro.
    pub attributes: u16,
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
        mc: MutationContext<'gc, '_>,
        this: ScriptObject<'gc>,
        fn_proto: Object<'gc>,
    ) -> Value<'gc> {
        let attributes = Attribute::from_bits_truncate(self.attributes);
        let value = match self.kind {
            DeclKind::Property { getter, setter } => {
                let getter =
                    FunctionObject::function(mc, Executable::Native(getter), fn_proto, fn_proto);
                let setter = setter.map(|setter| {
                    FunctionObject::function(mc, Executable::Native(setter), fn_proto, fn_proto)
                });
                this.add_property(mc, self.name.into(), getter, setter, attributes);
                return Value::Undefined;
            }
            DeclKind::Method(func) => {
                FunctionObject::bare_function(mc, Some(Executable::Native(func)), None, fn_proto)
                    .into()
            }
            DeclKind::Function(func) => {
                FunctionObject::function(mc, Executable::Native(func), fn_proto, fn_proto).into()
            }
            DeclKind::String(s) => s.into(),
            DeclKind::Bool(b) => b.into(),
            DeclKind::Int(i) => i.into(),
            DeclKind::Float(f) => f.into(),
        };

        this.define_value(mc, self.name, value, attributes);
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
            attributes: 0,
        }
    };
    (@__prop $kind:ident($name:literal $(,$args:expr)*; $($attributes:ident)|*$(; version($version:tt))?) ) => {
        crate::avm1::property_decl::Declaration {
            name: $name,
            kind: declare_properties!(@__kind $kind ($($args),*)),
            /*
                WARNING: HORRIBLE HACK AHEAD!

                To declare property attributes in a way that is valid in `const` context,
                we store them as raw `u8`s and do the bitflag management ourselves.

                Here are two better ways that unfortunately don't work.

                A) `Attribute::FOO | Attribute::BAR`

                This can't be used, because operator overloading doesn't work in `const` context.

                B) `Attributes::from_bits_truncate(Attribute::FOO.bits() | Attribute::BAR.bits())`

                Here, everything is a proper `const fn` and so this should work correctly,
                but NO!, we hit an ICE in the compiler :(
                See:
                    https://github.com/rust-lang/rust/issues/81899
                    https://github.com/rust-lang/rust/issues/84957

                TODO: use the `B)` desugaring once the above ICE is fixed.
            */
            attributes: 0 $(| declare_properties!(@__attr $attributes))* $(| declare_properties!(@__version $version))*,
        }
    };
    // MAKE SURE THESE VALUES ARE IN SYNC WITH THE `Attribute` DEFINITION!
    (@__attr DONT_ENUM) => {
        (1 << 0)
    };
    (@__attr DONT_DELETE) => {
        (1 << 1)
    };
    (@__attr READ_ONLY) => {
        (1 << 2)
    };
    (@__version 5) => {
        0b0000_0000_0000_0000
    };
    (@__version 6) => {
        0b0000_0000_1000_0000
    };
    (@__version 7) => {
        0b0000_0101_0000_0000
    };
    (@__version 8) => {
        0b0001_0000_0000_0000
    };
    (@__version 9) => {
        0b0010_0000_0000_0000
    };
    (@__version 10) => {
        0b0100_0000_0000_0000
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
