//! Special object that implements `super`

use core::fmt;

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::ExecutionReason;
use crate::avm1::object::{search_prototype, ExecutionName};
use crate::avm1::{NativeObject, Object, Value};
use crate::string::AvmString;
use gc_arena::Collect;
use ruffle_macros::istr;

/// Implementation of the `super` object in AS2.
///
/// A `SuperObject` references all data from another object, but with one layer
/// of prototyping removed. It's as if the given object had been constructed
/// with its parent class.
#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct SuperObject<'gc> {
    /// The object present as `this` throughout the superchain.
    this: Object<'gc>,

    /// The prototype depth of the currently-executing method.
    depth: u8,

    /// Adds a niche, so that enums contaning this type can use it for their discriminant.
    _niche: ruffle_common::utils::ZeroU8,
}

impl fmt::Debug for SuperObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SuperObject")
            .field("this", &self.this)
            .field("depth", &self.depth)
            .finish()
    }
}

impl<'gc> SuperObject<'gc> {
    /// Construct a `super` for an incoming stack frame.
    pub fn new(this: Object<'gc>, depth: u8) -> Self {
        Self {
            this,
            depth,
            _niche: Default::default(),
        }
    }

    pub fn this(&self) -> Object<'gc> {
        self.this
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    fn base_proto(&self, activation: &mut Activation<'_, 'gc>) -> Option<Object<'gc>> {
        let mut proto = self.this();
        for _ in 0..self.depth() {
            match proto.proto(activation) {
                Value::Object(p) => proto = p,
                _ => return None,
            }
        }
        Some(proto)
    }

    pub(super) fn proto(&self, activation: &mut Activation<'_, 'gc>) -> Value<'gc> {
        match self.base_proto(activation) {
            Some(p) => p.proto(activation),
            None => Value::Undefined,
        }
    }

    pub(super) fn call(
        &self,
        name: impl Into<ExecutionName<'gc>>,
        activation: &mut Activation<'_, 'gc>,
        args: &[Value<'gc>],
    ) -> Result<Value<'gc>, Error<'gc>> {
        let Some(proto) = self.base_proto(activation) else {
            return Ok(Value::Undefined);
        };

        let constructor = istr!("__constructor__");
        let Some((Value::Object(constructor), _depth)) =
            search_prototype(proto.into(), constructor, activation, proto, false)?
        else {
            return Ok(Value::Undefined);
        };

        let NativeObject::Function(constr) = constructor.native() else {
            return Ok(Value::Undefined);
        };

        constr.exec_constructor(
            name.into(),
            activation,
            self.this().into(),
            self.depth() + 1,
            args,
            ExecutionReason::ConstructorCall,
            constructor,
        )
    }

    pub(super) fn call_method(
        &self,
        name: AvmString<'gc>,
        args: &[Value<'gc>],
        activation: &mut Activation<'_, 'gc>,
        reason: ExecutionReason,
    ) -> Result<Value<'gc>, Error<'gc>> {
        let this = self.this();
        let Some((Value::Object(method), depth)) =
            search_prototype(self.proto(activation), name, activation, this, true)?
        else {
            return Ok(Value::Undefined);
        };

        match method.as_function() {
            Some(exec) => exec.exec(
                ExecutionName::Dynamic(name),
                activation,
                this.into(),
                self.depth() + depth + 1,
                args,
                reason,
                method,
            ),
            None => method.call(name, activation, this.into(), args),
        }
    }
}
