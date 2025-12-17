//! Declarative macro for defining AVM1 properties.

use gc_arena::Mutation;

use crate::avm1::function::{FunctionObject, NativeFunction, TableNativeFunction};
use crate::avm1::property::Attribute;
use crate::avm1::{Object, Value};
use crate::string::{HasStringContext, StringContext, WStr};

pub struct DeclContext<'a, 'gc> {
    pub strings: &'a mut StringContext<'gc>,
    pub object_proto: Object<'gc>,
    pub fn_proto: Object<'gc>,
}

impl<'gc> HasStringContext<'gc> for DeclContext<'_, 'gc> {
    fn strings_ref(&self) -> &StringContext<'gc> {
        self.strings
    }
}

impl<'gc> DeclContext<'_, 'gc> {
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.strings.gc()
    }

    #[inline(never)]
    pub fn define_properties_on(&mut self, this: Object<'gc>, decls: &[Declaration<'gc>]) {
        for decl in decls {
            decl.define_on(self.strings, this, self.fn_proto);
        }
    }

    pub fn empty_class(&self, super_proto: Object<'gc>) -> SystemClass<'gc> {
        let proto = Object::new(self.strings, Some(super_proto));
        let constr = FunctionObject::empty().build(self.strings, self.fn_proto, Some(proto));
        SystemClass { proto, constr }
    }

    /// Creates a class with a 'normal' constructor. This should be used for classes whose constructor
    /// is implemented in bytecode in Flash Player's `playerglobals.swf`.
    pub fn class(&self, function: NativeFunction, super_proto: Object<'gc>) -> SystemClass<'gc> {
        let proto = Object::new(self.strings, Some(super_proto));
        let constr =
            FunctionObject::native(function).build(self.strings, self.fn_proto, Some(proto));
        SystemClass { proto, constr }
    }

    /// Creates a class with a 'special' constructor. This should be used for classes with a native
    /// constructor in Flash Player's `playerglobals.swf`.
    pub fn native_class(
        &self,
        constructor: NativeFunction,
        function: Option<NativeFunction>,
        super_proto: Object<'gc>,
    ) -> SystemClass<'gc> {
        let proto = Object::new(self.strings, Some(super_proto));
        Self::native_class_with_proto(self, constructor, function, proto)
    }

    pub fn native_class_with_proto(
        &self,
        constructor: NativeFunction,
        function: Option<NativeFunction>,
        proto: Object<'gc>,
    ) -> SystemClass<'gc> {
        let constr = FunctionObject::constructor(constructor, function).build(
            self.strings,
            self.fn_proto,
            Some(proto),
        );
        SystemClass { proto, constr }
    }
}

#[derive(Copy, Clone)]
pub struct SystemClass<'gc> {
    pub proto: Object<'gc>,
    pub constr: Object<'gc>,
}

/// A list of static [`Declaration`]s.
///
/// Morally, this is `&'static [for<'gc> Declaration<'gc>]`, but due to Rust's limitations
/// we need to emulate it using a function type. Rust also requires the `'gc` lifetime to be
/// mentioned in the arguments, which is why the `fn` takes a `DeclContext` even though it
/// isn't useful at runtime.
pub type StaticDeclarations = for<'gc> fn(&DeclContext<'_, 'gc>) -> &'gc [Declaration<'gc>];

/// The declaration of a property, method, or simple field, that
/// can be defined on a [`Object`].
#[derive(Copy, Clone)]
pub struct Declaration<'gc> {
    pub name: &'static [u8],
    pub kind: DeclKind<'gc>,
    pub attributes: Attribute,
}

/// All the possible types of a [`Declaration`].
#[derive(Copy, Clone)]
pub enum DeclKind<'gc> {
    /// Declares a property with a getter and an optional setter.
    TableProperty {
        native: TableNativeFunction,
        getter: u16,
        setter: Option<u16>,
    },
    Property {
        getter: NativeFunction,
        setter: Option<NativeFunction>,
    },
    /// Declares a native host function.
    ///
    /// This is intended for use with defining host object prototypes. Notably,
    /// this creates a function object without an explicit `prototype`, which
    /// is only possible when defining host functions.
    TableMethod(TableNativeFunction, u16),
    Method(NativeFunction),
    /// Declares a native function with a `prototype`.
    /// Prefer using [`Self::Method`] when defining host functions.
    #[expect(unused)] // kept for symmetry.
    TableFunction(TableNativeFunction, u16),
    Function(NativeFunction),
    /// Declares a static string value.
    String(&'static [u8]),
    /// Declares a static bool value.
    Bool(bool),
    /// Declares a static int value.
    Int(i32),
    /// Declares a static float value.
    Float(f64),
    /// Declares an object value (can't be used in static contexts).
    Object(Object<'gc>),
}

impl<'gc> Declaration<'gc> {
    #[inline(never)]
    /// Defines the field represented by this declaration on a [`Object`].
    /// Returns the value defined on the object, or `undefined` if this declaration
    /// defined a property.
    pub fn define_on(
        &self,
        context: &mut StringContext<'gc>,
        this: Object<'gc>,
        fn_proto: Object<'gc>,
    ) -> Value<'gc> {
        let mc = context.gc();

        let name = context.intern_static(WStr::from_units(self.name));
        let value = match self.kind {
            DeclKind::Property { getter, setter } => {
                // Property objects are unobservable by user code, so a bare function is enough.
                let getter = FunctionObject::native(getter).build(context, fn_proto, None);
                let setter = setter
                    .map(|setter| FunctionObject::native(setter).build(context, fn_proto, None));
                this.add_property(mc, name.into(), getter, setter, self.attributes);
                return Value::Undefined;
            }
            DeclKind::TableProperty {
                native,
                getter,
                setter,
            } => {
                // Property objects are unobservable by user code, so a bare function is enough.
                let getter =
                    FunctionObject::table_native(native, getter).build(context, fn_proto, None);
                let setter = setter.map(|setter| {
                    FunctionObject::table_native(native, setter).build(context, fn_proto, None)
                });
                this.add_property(mc, name.into(), getter, setter, self.attributes);
                return Value::Undefined;
            }
            DeclKind::Method(f) | DeclKind::Function(f) => {
                let p = matches!(self.kind, DeclKind::Function(_)).then_some(fn_proto);
                FunctionObject::native(f).build(context, fn_proto, p).into()
            }
            DeclKind::TableMethod(f, index) | DeclKind::TableFunction(f, index) => {
                let p = matches!(self.kind, DeclKind::Function(_)).then_some(fn_proto);
                FunctionObject::table_native(f, index)
                    .build(context, fn_proto, p)
                    .into()
            }
            DeclKind::String(s) => context.intern_static(WStr::from_units(s)).into(),
            DeclKind::Bool(b) => b.into(),
            DeclKind::Int(i) => i.into(),
            DeclKind::Float(f) => f.into(),
            DeclKind::Object(o) => o.into(),
        };

        this.define_value(mc, name, value, self.attributes);
        value
    }
}

/// Declares a list of property [`Declaration`]s that can be later defined on [`Object`]s.
///
/// # Usage:
///
/// ```rust,ignore
/// const DECLS: StaticDeclarations = declare_static_properties! {
///     "length" => property(get_length);
///     "filters" => property(get_filters, set_filters);
///     "to_string" => method(to_string);
///     "to_string2" => function(to_string);
///     // switches to 'table mode': function-like definitions will now take
///     // an integer index instead of a function pointer, and will dispatch it to the
///     // method provided here.
///     use fn method;
///     "callme" => function(CALLME);
///     // you can go back to the 'default' mode
///     use default;
///     "locale" => string("en-US");
///     "enabled" => bool(true);
///     "size" => int(123);
///     "scale" => float(0.85);
///     // all declarations can also specify attributes
///     "hidden" => string("shh!"; DONT_ENUM | DONT_DELETE | READ_ONLY);
/// };
///
/// mod method {
///   pub const CALLME: u16 = 0;
/// }
///
/// fn method(..., id: u16) -> Result<Value<'gc>, Error<'gc>> {
///   match id {
///     CALLME => { ... }
///     _ => Ok(Value::Undefined)
///   }
/// }
/// ```
macro_rules! declare_static_properties {
    ( $($tts:tt)* ) => {
        |_: &$crate::avm1::property_decl::DeclContext<'_, '_>| const { declare_properties!($($tts)*) }
    }
}

/// Like `declare_static_properties`, but can be used outside of const contexts.
macro_rules! declare_properties {
    ( $($tts:tt)* ) => {{
        const fn __assert_ascii(s: &str) -> &[u8] {
            assert!(s.is_ascii());
            s.as_bytes()
        }

        __declare_properties!(@stmt [default] [/* out */] $($tts)*)
    }};
}

// Internal implementation
macro_rules! __declare_properties {
    // Main TT-muncher loop for distinguishing between `use ...` and `"name" => ...`
    // and for threading the current 'use' mode.
    (@stmt [$($mode:tt)*] [$($out:tt)*] )=> {
        &[ $($out)* ]
    };
    (
        @stmt [$($mode:tt)*]
        [$($out:tt)*]
        $name:literal => $kind:ident($($args:tt)*);
        $($rest:tt)*
    ) => {
        __declare_properties!(
            @stmt [$($mode)*]
            [ $($out)* __declare_properties!(
                @prop [$($mode)*] $name $kind [/* args out */] $($args)*
            ), ]
            $($rest)*
        )
    };
    (
        @stmt [$($mode:tt)*]
        [$($out:tt)*]
        use default;
        $($rest:tt)*
    ) => {
        __declare_properties!(@stmt [default] [ $($out)* ] $($rest)*)
    };
    (
        @stmt [$($mode:tt)*]
        [$($out:tt)*]
        use fn $($path:tt)::+;
        $($rest:tt)*
    ) => {
        __declare_properties!(@stmt [fn $($path)::+] [ $($out)* ] $($rest)*)
    };

    // Property args TT-muncher loop: we want to parse until the first ';'.
    // The $args need to be kept as tt's all the way to the end, where they
    // will be matched as an expr or an ident depending on the $mode.
    (
        @prop [$($mode:tt)*] $name:literal $kind:ident
        [$($args:tt)*] $(; $($attributes:ident)|*)?
    ) => {
        $crate::avm1::property_decl::Declaration {
            name: const { __assert_ascii($name) },
            kind: __declare_properties!(@kind [$($mode)*] $kind ($($args)*)),
            attributes: $crate::avm1::property::Attribute::from_bits_truncate(
                0 $($(| $crate::avm1::property::Attribute::$attributes.bits())*)?
            ),
        }
    };
    (
        @prop [$($mode:tt)*] $name:literal $kind:ident
        [$($args:tt)*] $tt:tt $($rest:tt)*
    ) => {
        __declare_properties!(
            @prop [$($mode)*] $name $kind [$($args)* $tt] $($rest)*
        )
    };

    // The various kinds of declarations.
    (@kind [default] property($getter:expr, $setter:expr)) => {
        $crate::avm1::property_decl::DeclKind::Property {
            getter: $getter,
            setter: Some($setter),
        }
    };
    (@kind [default] property($getter:expr)) => {
        $crate::avm1::property_decl::DeclKind::Property {
            getter: $getter,
            setter: None,
        }
    };
    (@kind [fn $($path:ident)::+] property($getter:ident, $setter:ident)) => {
        $crate::avm1::property_decl::DeclKind::TableProperty {
            native: $($path)::+,
            getter: $($path::)+$getter,
            setter: Some($($path::)+$setter),
        }
    };
    (@kind [fn $($path:ident)::+] property($getter:ident)) => {
        $crate::avm1::property_decl::DeclKind::TableProperty {
            native: $($path)::+,
            getter: $($path::)+$getter,
            setter: None,
        }
    };
    (@kind [default] method($method:expr)) => {
        $crate::avm1::property_decl::DeclKind::Method($method)
    };
    (@kind [fn $($path:ident)::+] method($method:ident)) => {
        $crate::avm1::property_decl::DeclKind::TableMethod($($path)::+, $($path::)+$method)
    };
    (@kind [default] function($function:expr)) => {
        $crate::avm1::property_decl::DeclKind::Function($function)
    };
    (@kind [fn $($path:ident)::+] function($function:ident)) => {
        $crate::avm1::property_decl::DeclKind::TableFunction($($path)::+, $($path::)+$function)
    };
    (@kind $_mode:tt string($string:expr)) => {
        $crate::avm1::property_decl::DeclKind::String(const { __assert_ascii($string) })
    };
    (@kind $_mode:tt bool($boolean:expr)) => {
        $crate::avm1::property_decl::DeclKind::Bool($boolean)
    };
    (@kind $_mode:tt int($int:expr)) => {
        $crate::avm1::property_decl::DeclKind::Int($int)
    };
    (@kind $_mode:tt float($float:expr)) => {
        $crate::avm1::property_decl::DeclKind::Float($float)
    };
    (@kind $_mode:tt object($obj:expr)) => {
        $crate::avm1::property_decl::DeclKind::Object($obj)
    };
}

macro_rules! table_constructor {
    ($($method:ident)::+) => {
        table_constructor!($($method)::*, CONSTRUCTOR)
    };
    ($($method:ident)::+, $index:ident) => {
        // TODO: add support to ASnative-style table constructors to FunctionObject.
        |activation, this, args| $($method)::+(activation, this, args, ($($method::)+$index))
    };
}
