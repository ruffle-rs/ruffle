use crate::avm1::function::Executable;
use crate::avm1::property::Attribute;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, ObjectPtr, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use crate::display_object::DisplayObject;

use enumset::EnumSet;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::hash_map::RandomState;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Collect)]
#[collect(no_drop)]
pub struct ArrayObject<'gc>(GcCell<'gc, ArrayObjectData<'gc>>);

#[derive(Debug, Collect)]
#[collect(no_drop)]
pub struct ArrayObjectData<'gc> {
    base: ScriptObject<'gc>,
}

fn get_length<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(this.get_length().into())
}

impl<'gc> ArrayObject<'gc> {
    pub fn array(
        gc_context: MutationContext<'gc, '_>,
        proto: Option<Object<'gc>>,
    ) -> ArrayObject<'gc> {
        let base = ScriptObject::object(gc_context, proto);
        base.add_property(
            gc_context,
            "length",
            Executable::Native(get_length),
            None,
            Attribute::DontDelete | Attribute::DontEnum,
        );

        ArrayObject(GcCell::allocate(gc_context, ArrayObjectData { base }))
    }
}

impl<'gc> TObject<'gc> for ArrayObject<'gc> {
    fn get_local(
        &self,
        name: &str,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().base.get_local(name, avm, context, this)
    }

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        if let Ok(index) = name.parse::<i32>() {
            if index >= self.get_length() {
                self.set_length(context.gc_context, index + 1);
            }
        } else if name == "length" {
            self.set_length(
                context.gc_context,
                value
                    .as_number(avm, context)
                    .map(|v| v.abs() as i32)
                    .unwrap_or(0),
            );
        }
        self.0.read().base.set(name, value, avm, context)
    }

    fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().base.call(avm, context, this, args)
    }

    #[allow(clippy::new_ret_no_self)]
    fn new(
        &self,
        _avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
        _args: &[Value<'gc>],
    ) -> Result<Object<'gc>, Error> {
        Ok(ArrayObject::array(context.gc_context, Some(this)).into())
    }

    fn delete(&self, gc_context: MutationContext<'gc, '_>, name: &str) -> bool {
        self.0.read().base.delete(gc_context, name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().base.proto()
    }

    fn define_value(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        value: Value<'gc>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .define_value(gc_context, name, value, attributes)
    }

    fn add_property(
        &self,
        gc_context: MutationContext<'gc, '_>,
        name: &str,
        get: Executable<'gc>,
        set: Option<Executable<'gc>>,
        attributes: EnumSet<Attribute>,
    ) {
        self.0
            .read()
            .base
            .add_property(gc_context, name, get, set, attributes)
    }

    fn has_property(&self, name: &str) -> bool {
        self.0.read().base.has_property(name)
    }

    fn has_own_property(&self, name: &str) -> bool {
        self.0.read().base.has_own_property(name)
    }

    fn is_property_overwritable(&self, name: &str) -> bool {
        self.0.read().base.is_property_overwritable(name)
    }

    fn is_property_enumerable(&self, name: &str) -> bool {
        self.0.read().base.is_property_enumerable(name)
    }

    fn get_keys(&self) -> HashSet<String, RandomState> {
        self.0.read().base.get_keys()
    }

    fn get_length(&self) -> i32 {
        self.0.read().base.get_length()
    }

    fn set_length(&self, gc_context: MutationContext<'gc, '_>, length: i32) {
        self.0.read().base.set_length(gc_context, length)
    }

    fn as_string(&self) -> String {
        self.0.read().base.as_string()
    }

    fn type_of(&self) -> &'static str {
        self.0.read().base.type_of()
    }

    fn as_script_object(&self) -> Option<ScriptObject<'gc>> {
        self.0.read().base.as_script_object()
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        self.0.read().base.as_display_object()
    }

    fn as_executable(&self) -> Option<Executable<'gc>> {
        self.0.read().base.as_executable()
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.read().base.as_ptr()
    }
}
