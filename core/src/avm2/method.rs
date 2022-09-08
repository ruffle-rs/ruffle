//! AVM2 methods

use crate::avm2::activation::Activation;
use crate::avm2::object::Object;
use crate::avm2::ClassObject;
use crate::avm2::script::TranslationUnit;
use crate::avm2::value::{abc_default_value, Value};
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::string::AvmString;
use gc_arena::{Collect, CollectionContext, Gc, MutationContext};
use std::borrow::Cow;
use std::cell::Cell;
use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;
use swf::avm2::types::{
    AbcFile, Index, Method as AbcMethod, MethodBody as AbcMethodBody,
    MethodFlags as AbcMethodFlags, MethodParam as AbcMethodParam,
};

/// Represents a function defined in Ruffle's code.
///
/// Parameters are as follows:
///
///  * The AVM2 runtime
///  * The action context
///  * The current `this` object
///  * The arguments this function was called with
///
/// Native functions are allowed to return a value or `None`. `None` indicates
/// that the given value will not be returned on the stack and instead will
/// resolve on the AVM stack, as if you had called a non-native function. If
/// your function yields `None`, you must ensure that the top-most activation
/// in the AVM1 runtime will return with the value of this function.
pub type NativeMethodImpl = for<'gc> fn(
    &mut Activation<'_, 'gc, '_>,
    Option<Object<'gc>>,
    &[Value<'gc>],
) -> Result<Value<'gc>, Error>;

/// Configuration of a single parameter of a method.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ParamConfig<'gc> {
    /// The name of the parameter.
    pub param_name: AvmString<'gc>,

    /// The name of the type of the parameter.
    pub param_type_name: Multiname<'gc>,

    /// The default value for this parameter.
    pub default_value: Option<Value<'gc>>,
}

impl<'gc> ParamConfig<'gc> {
    fn from_abc_param(
        config: &AbcMethodParam,
        txunit: TranslationUnit<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let param_name = if let Some(name) = &config.name {
            txunit.pool_string(name.0, activation.context.gc_context)?
        } else {
            "<Unnamed Parameter>".into()
        };
        let param_type_name = txunit
            .pool_multiname_static_any(config.kind, activation.context.gc_context)?
            .deref()
            .clone();

        let default_value = if let Some(dv) = &config.default_value {
            Some(abc_default_value(txunit, dv, activation)?)
        } else {
            None
        };

        Ok(Self {
            param_name,
            param_type_name,
            default_value,
        })
    }

    pub fn of_type(name: impl Into<AvmString<'gc>>, param_type_name: Multiname<'gc>) -> Self {
        Self {
            param_name: name.into(),
            param_type_name,
            default_value: None,
        }
    }

    pub fn optional(
        name: impl Into<AvmString<'gc>>,
        param_type_name: Multiname<'gc>,
        default_value: impl Into<Value<'gc>>,
    ) -> Self {
        Self {
            param_name: name.into(),
            param_type_name,
            default_value: Some(default_value.into()),
        }
    }
}

/// Represents a reference to an AVM2 method and body.
#[derive(Collect, Clone, Debug)]
#[collect(no_drop)]
pub struct BytecodeMethod<'gc> {
    /// The translation unit this function was defined in.
    pub txunit: TranslationUnit<'gc>,

    /// The underlying ABC file of the above translation unit.
    #[collect(require_static)]
    pub abc: Rc<AbcFile>,

    /// The ABC method this function uses.
    pub abc_method: u32,

    /// The ABC method body this function uses.
    pub abc_method_body: Option<u32>,

    pub try_optimize: Cell<bool>,
    pub optimized_method_body: RefCell<Option<AbcMethodBody>>,

    /// The parameter signature of this method.
    pub signature: Vec<ParamConfig<'gc>>,

    /// The return type of this method.
    pub return_type: Multiname<'gc>,

    /// Whether or not this method was declared as a free-standing function.
    ///
    /// A free-standing function corresponds to the `Function` trait type, and
    /// is instantiated with the `newfunction` opcode.
    pub is_function: bool,
}

impl<'gc> BytecodeMethod<'gc> {
    /// Construct an `BytecodeMethod` from an `AbcFile` and method index.
    pub fn from_method_index(
        txunit: TranslationUnit<'gc>,
        abc_method: Index<AbcMethod>,
        is_function: bool,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<Self, Error> {
        let abc = txunit.abc();
        let mut signature = Vec::new();

        if abc.methods.get(abc_method.0 as usize).is_some() {
            let method = &abc.methods[abc_method.0 as usize];
            for param in &method.params {
                signature.push(ParamConfig::from_abc_param(param, txunit, activation)?);
            }

            let return_type = txunit
                .pool_multiname_static_any(method.return_type, activation.context.gc_context)?
                .deref()
                .clone();

            for (index, method_body) in abc.method_bodies.iter().enumerate() {
                if method_body.method.0 == abc_method.0 {
                    return Ok(Self {
                        txunit,
                        abc: txunit.abc(),
                        abc_method: abc_method.0,
                        abc_method_body: Some(index as u32),
                        try_optimize: Cell::new(true),
                        optimized_method_body: RefCell::new(None),
                        signature,
                        return_type,
                        is_function,
                    });
                }
            }
        }

        Ok(Self {
            txunit,
            abc: txunit.abc(),
            abc_method: abc_method.0,
            abc_method_body: None,
            try_optimize: Cell::new(true),
            optimized_method_body: RefCell::new(None),
            signature,
            return_type: Multiname::any(),
            is_function,
        })
    }

    /// Get the underlying ABC file.
    pub fn abc(&self) -> Rc<AbcFile> {
        self.txunit.abc()
    }

    /// Get the underlying translation unit this method was defined in.
    pub fn translation_unit(&self) -> TranslationUnit<'gc> {
        self.txunit
    }

    /// Get a reference to the ABC method entry this refers to.
    pub fn method(&self) -> &AbcMethod {
        self.abc.methods.get(self.abc_method as usize).unwrap()
    }

    /// Get a reference to the ABC method body entry this refers to.
    ///
    /// Some methods do not have bodies; this returns `None` in that case.
    pub fn body(&self) -> Option<&AbcMethodBody> {
        if let Some(abc_method_body) = self.abc_method_body {
            self.abc.method_bodies.get(abc_method_body as usize)
        } else {
            None
        }
    }

    #[inline(never)]
    pub fn optimize(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        this_class: Option<ClassObject<'gc>>,
    ) {
        let this_class = if let Some(cls) = this_class { cls } else { return; };
        if let Some(this) = activation.this() {
            if !this.is_of_type(this_class, activation) { return; } // it's a static method
        }

        let body = self.body().unwrap();

        let mut new_body = body.clone(); // todo skip cloning vec?
        new_body.code.clear();
        let mut writer = Writer::new(&mut new_body.code);

        use swf::avm2::read::Reader;
        use swf::avm2::write::Writer;
        use swf::avm2::types::Op;
        use crate::avm2::property::Property;
        use crate::avm2::object::TObject;
        use crate::avm2::property::resolve_class_private;
        use crate::avm2::property::PropertyClass;
        use crate::avm2::property::ResolveOutcome;

        // idea:
        // - record all jump targets (basic block edges)
        // - create a set of valid optimizable locals
        //   - local is optimizable if:
        //     - its setLocal is always preceded by the same cast (and is not a jump target)
        //     - if it's `this` (in an instance method) or typed argument, its type also needs to match
        // - for every getlocal->get/set/init/callproperty pair:
        //   - the *property op can be optimized if:
        //     - local is a valid optimizable local
        //     - the pair doesn't cross a jump target
        //     - the *property op multiname is a static multiname
        //     - the lookup succeeds in a valid replacement
        //     - do a replacement:
        //       - get/set/initproperty on slot -> get/set/init(?)slot
        //       - get/set/initproperty on getter -> callmethod
        //       - callproperty on method -> callmethod
        //   - if the getproperty pair is followed by another *property:
        //     - we can grab the property's type from `slot_classes`
        //     - (again, need to make sure this doesn't cross a jump target)

        // questions:
        //  - are we sure initproperty ops behave the same? Maybe they just get optimized to setslot?

        // missing parts in the current implementation:
        // - anything regarding basic blocks
        //   - more careful validation
        //   - within basic blocks, we could track variables that are set to typed value and immediately
        //     used, even if they aren't typed consistently across entire function, or not at all
        // - a lot of checks end in a panic - they aren't supposed to be generated
        //   by "normal" compilers so MVP treated them as sanity checks,
        //   but generally they are supposed to disable optimizations for the local
        //   (one example is a `coerce->setlocal` to a different type than variable's previous type)
        // - set/initproperty
        // - can there be holes in bytecode? Can we safely read_op() in a loop?

        // longer term:
        // - fancier patterns (`++this.thing`)
        // - support optimizing in static methods?
        //   - this might require ClassObject/vtable refactor
        // - pretty sure this is also a good place for verification
        // - refactor to not rely on current Activation state
        //   - (it's extremely ugly that we grab `this_class` from activation
        //      instead from method information itself,
        //      and use runtime type of `this` to check if method is static)
        // - allow generating smaller/bigger opcodes
        //   - this requires recalculating all jump targets
        //   - alternatively... just don't write bytecode at all and store Vec<Op>
        //     this would remove read_op() overhead entirely

        pub fn encoded_u30_len(mut n: u32) -> u32 {
            // needed to calculate if new opcode will fit in space of previous opcode
            let mut result = 0;
            loop {
                n >>= 7;
                result += 1;
                if n == 0 {
                    break;
                }
            }
            result
        }

        let mut local_types: Vec<Option<ClassObject<'gc>>> = vec![None; body.num_locals as usize];
        local_types[0] = Some(this_class);

        let signature = self.signature();
        for (i, param) in signature.iter().enumerate() {
            let param_type = activation.resolve_type(&param.param_type_name).unwrap();
            local_types[i+1] = param_type;
        }

        let mut current_type: Option<ClassObject<'gc>> = None;
        let mut reader = Reader::new(&body.code);
        while let Ok(op) = reader.read_op() {
            if let Op::Coerce { index: coerce_index } = op {
                let multiname = self
                    .translation_unit()
                    .pool_maybe_uninitialized_multiname(coerce_index, activation.context.gc_context).unwrap();
                if !multiname.has_lazy_component() {
                    let cls = activation.resolve_type(&multiname).unwrap();
                    current_type = cls;
                }
            } else if let Op::SetLocal { index: local_index } = op {
                let local_index = local_index as usize;

                if local_index < signature.len() + 1 {
                    if local_types[local_index] != current_type {
                        // todo: does verifier accept code like this?
                        // if it does, we should just deoptimize (set type to None)
                        panic!("arg {:?} does not match {:?}", local_types[local_index], current_type);
                    }
                } else {
                    if local_types[local_index].is_some() && local_types[local_index] != current_type {
                        // todo: does verifier accept code like this?
                        // if it does, we should just deoptimize (set type to None)
                        panic!("local {:?} does not match {:?}", local_types[local_index], current_type);
                    }
                    local_types[local_index] = current_type;
                }
                current_type = None;
            }
        }

        let mut current_type: Option<ClassObject<'gc>> = None;
        let mut reader = Reader::new(&body.code);
        while let Ok(mut op) = reader.read_op() {
            let mut old_len = 0;
            let mut new_len = 0;
            if let Op::GetLocal { index: local_index } = op {
                current_type = local_types[local_index as usize];
            } else if let Op::GetProperty { index: name_index } = op {
                let cls = current_type;
                current_type = None;
                if let Some(cls) = cls {
                    let multiname = self
                        .translation_unit()
                        .pool_maybe_uninitialized_multiname(name_index, activation.context.gc_context).unwrap();
                    if !multiname.has_lazy_component() {
                        match cls.instance_vtable().get_trait(&multiname) {
                            Some(Property::Slot { slot_id }) | Some(Property::ConstSlot { slot_id }) => {
                                // we need to fit in previous opcode's encoding
                                // thankfully it's true most of the time
                                let name_len = encoded_u30_len(name_index.0);
                                let slot_len = encoded_u30_len(slot_id);
                                if slot_len <= name_len {
                                    // todo: I hate this :D
                                    // slot_classes _really_ shouldn't be this lazy,
                                    // they should all be ready by the time class is instantiated, if not earlier
                                    let new_cls = cls.instance_vtable().slot_classes()[slot_id as usize].clone();
                                    match new_cls {
                                        PropertyClass::Class(class) => { current_type = Some(class); },
                                        PropertyClass::Name(gc) => {
                                            let (name, unit) = &*gc;
                                            let outcome = resolve_class_private(name, *unit, activation).unwrap();
                                            if let ResolveOutcome::Class(class) = outcome {
                                                current_type = Some(class);
                                            }
                                        }
                                        PropertyClass::Any => {},
                                    }

                                    old_len = name_len;
                                    new_len = slot_len;
                                    op = Op::GetSlot { index: slot_id };
                                }
                            },
                            Some(Property::Virtual { get: Some(get), .. }) => {
                                let name_len = encoded_u30_len(name_index.0);
                                let zero_len = encoded_u30_len(0);
                                let meth_id_len = encoded_u30_len(get);
                                // this usually works as often name_id > 127
                                // but meth_id < 127 to it just about fits
                                if name_len >= zero_len + meth_id_len {
                                    old_len = name_len;
                                    new_len = zero_len + meth_id_len;
                                    op = Op::CallMethod { num_args: 0, index: Index::new(get) };
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                current_type = None;
            }

            writer.write_op(&op).expect("failed to write???");
            if old_len > new_len {
                for _ in 0..(old_len-new_len) {
                    writer.write_op(&Op::Nop).expect("failed to write???");
                }
            }
        }

        if body.code.len() != new_body.code.len() {
            panic!("size mismatch");
        }

        *self.optimized_method_body.borrow_mut() = Some(new_body);
    }

    /// Get the list of method params for this method.
    pub fn signature(&self) -> &[ParamConfig<'gc>] {
        &self.signature
    }

    /// Get the name of this method.
    pub fn method_name(&self) -> &str {
        let name_index = self.method().name.0 as usize;
        if name_index == 0 {
            return "";
        }

        self.abc
            .constant_pool
            .strings
            .get(name_index - 1)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Determine if a given method is variadic.
    ///
    /// Variadic methods shove excess parameters into a final register.
    pub fn is_variadic(&self) -> bool {
        self.method()
            .flags
            .intersects(AbcMethodFlags::NEED_ARGUMENTS | AbcMethodFlags::NEED_REST)
    }

    /// Determine if a given method is unchecked.
    ///
    /// A method is unchecked if all of the following are true:
    ///
    ///  * The method was declared as a free-standing function
    ///  * The function does not use rest-parameters
    ///  * The function's parameters have no declared types or default values
    pub fn is_unchecked(&self) -> bool {
        if !self.is_function {
            return false;
        }

        for param in self.signature() {
            if !param.param_type_name.is_any() || param.default_value.is_some() {
                return false;
            }
        }

        !self.method().flags.contains(AbcMethodFlags::NEED_REST)
    }
}

/// An uninstantiated method
#[derive(Clone)]
pub struct NativeMethod<'gc> {
    /// The function to call to execute the method.
    pub method: NativeMethodImpl,

    /// The name of the method.
    pub name: Cow<'static, str>,

    /// The parameter signature of the method.
    pub signature: Vec<ParamConfig<'gc>>,

    /// Whether or not this method accepts parameters beyond those
    /// mentioned in the parameter list.
    pub is_variadic: bool,
}

unsafe impl<'gc> Collect for NativeMethod<'gc> {
    fn trace(&self, cc: CollectionContext) {
        self.signature.trace(cc);
    }
}

impl<'gc> fmt::Debug for NativeMethod<'gc> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeMethod")
            .field("method", &format!("{:p}", &self.method))
            .field("name", &self.name)
            .field("signature", &self.signature)
            .field("is_variadic", &self.is_variadic)
            .finish()
    }
}

/// An uninstantiated method that can either be natively implemented or sourced
/// from an ABC file.
#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub enum Method<'gc> {
    /// A native method.
    Native(Gc<'gc, NativeMethod<'gc>>),

    /// An ABC-provided method entry.
    Bytecode(Gc<'gc, BytecodeMethod<'gc>>),
}

impl<'gc> From<Gc<'gc, BytecodeMethod<'gc>>> for Method<'gc> {
    fn from(bm: Gc<'gc, BytecodeMethod<'gc>>) -> Self {
        Self::Bytecode(bm)
    }
}

impl<'gc> Method<'gc> {
    /// Define a builtin method with a particular param configuration.
    pub fn from_builtin_and_params(
        method: NativeMethodImpl,
        name: impl Into<Cow<'static, str>>,
        signature: Vec<ParamConfig<'gc>>,
        is_variadic: bool,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        Self::Native(Gc::allocate(
            mc,
            NativeMethod {
                method,
                name: name.into(),
                signature,
                is_variadic,
            },
        ))
    }

    /// Define a builtin with no parameter constraints.
    pub fn from_builtin(
        method: NativeMethodImpl,
        name: impl Into<Cow<'static, str>>,
        mc: MutationContext<'gc, '_>,
    ) -> Self {
        Self::Native(Gc::allocate(
            mc,
            NativeMethod {
                method,
                name: name.into(),
                signature: Vec::new(),
                is_variadic: true,
            },
        ))
    }

    /// Access the bytecode of this method.
    ///
    /// This function returns `Err` if there is no bytecode for this method.
    pub fn into_bytecode(self) -> Result<Gc<'gc, BytecodeMethod<'gc>>, Error> {
        match self {
            Method::Native { .. } => {
                Err("Attempted to unwrap a native method as a user-defined one".into())
            }
            Method::Bytecode(bm) => Ok(bm),
        }
    }

    /// Check if this method needs `arguments`.
    pub fn needs_arguments_object(&self) -> bool {
        match self {
            Method::Native { .. } => false,
            Method::Bytecode(bm) => bm.method().flags.contains(AbcMethodFlags::NEED_ARGUMENTS),
        }
    }
}
