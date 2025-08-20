//! Super op support

use crate::avm2::activation::Activation;
use crate::avm2::error::{self, reference_error, Error};
use crate::avm2::function::{exec, FunctionArgs};
use crate::avm2::multiname::Multiname;
use crate::avm2::object::{ClassObject, FunctionObject, TObject};
use crate::avm2::property::Property;
use crate::avm2::value::Value;

/// Like `Value::call_property`, but uses the instance vtable of `superclass`
/// for property lookups.
pub fn call_super<'gc>(
    superclass: ClassObject<'gc>,
    multiname: &Multiname<'gc>,
    receiver: Value<'gc>,
    arguments: FunctionArgs<'_, 'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let property = superclass.instance_vtable().get_trait(multiname);
    match property {
        Some(Property::Slot { slot_id }) | Some(Property::ConstSlot { slot_id }) => {
            let arguments = &arguments.to_slice();

            // Only objects can have slots
            let receiver_obj = receiver.as_object().unwrap();

            let func = receiver_obj.get_slot(slot_id);
            func.call(activation, receiver, arguments)
        }
        Some(Property::Method { disp_id }) => {
            call_method_super(activation, superclass, receiver, disp_id, arguments)
        }
        Some(Property::Virtual { get: Some(get), .. }) => {
            // Call the getter, then `Value::call` the result
            let obj =
                call_method_super(activation, superclass, receiver, get, FunctionArgs::empty())?;

            let arguments = &arguments.to_slice();
            obj.call(activation, receiver.into(), arguments)
        }
        Some(Property::Virtual { get: None, .. }) => Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::ReadFromWriteOnly,
            multiname,
            superclass.inner_class_definition(),
        )),
        None => {
            let qualified_multiname_name = multiname.as_uri(activation.strings());
            let qualified_class_name = superclass
                .inner_class_definition()
                .name()
                .to_qualified_name_err_message(activation.gc());

            return Err(Error::avm_error(reference_error(
                activation,
                &format!(
                    "Error #1070: Method {qualified_multiname_name} not found on {qualified_class_name}",
                ),
                1070,
            )?));
        }
    }
}

/// Like `Value::get_property`, but uses the instance vtable of `superclass`
/// for property lookups.
pub fn get_super<'gc>(
    superclass: ClassObject<'gc>,
    multiname: &Multiname<'gc>,
    receiver: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let property = superclass.instance_vtable().get_trait(multiname);

    match property {
        Some(Property::Slot { slot_id }) | Some(Property::ConstSlot { slot_id }) => {
            // Only objects can have slots
            let receiver = receiver.as_object().unwrap();

            Ok(receiver.get_slot(slot_id))
        }
        Some(Property::Method { disp_id }) => {
            let full_method = superclass
                .instance_vtable()
                .get_full_method(disp_id)
                .unwrap();
            let callee = FunctionObject::from_method(
                activation,
                full_method.method,
                full_method.scope(),
                Some(receiver),
                full_method.super_class_obj,
                Some(full_method.class),
            );

            Ok(callee.into())
        }
        Some(Property::Virtual {
            get: Some(disp_id), ..
        }) => call_method_super(
            activation,
            superclass,
            receiver,
            disp_id,
            FunctionArgs::empty(),
        ),
        Some(Property::Virtual { get: None, .. }) => Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::ReadFromWriteOnly,
            multiname,
            superclass.inner_class_definition(),
        )),
        None => Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::InvalidRead,
            multiname,
            superclass.inner_class_definition(),
        )),
    }
}

/// Like `Value::set_property`, but uses the instance vtable of `superclass`
/// for property lookups.
pub fn set_super<'gc>(
    superclass: ClassObject<'gc>,
    multiname: &Multiname<'gc>,
    value: Value<'gc>,
    receiver: Value<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<(), Error<'gc>> {
    let property = superclass.instance_vtable().get_trait(multiname);
    match property {
        Some(Property::Slot { slot_id }) => {
            // Only objects can have slots
            let receiver = receiver.as_object().unwrap();

            receiver.set_slot(slot_id, value, activation)
        }
        Some(Property::Method { .. }) => Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::AssignToMethod,
            multiname,
            superclass.inner_class_definition(),
        )),
        Some(Property::Virtual {
            set: Some(disp_id), ..
        }) => {
            let args = FunctionArgs::AsArgSlice {
                arguments: &[value],
            };

            call_method_super(activation, superclass, receiver, disp_id, args)?;

            Ok(())
        }
        Some(Property::ConstSlot { .. }) | Some(Property::Virtual { set: None, .. }) => {
            if activation.is_interpreter() {
                Err(error::make_reference_error(
                    activation,
                    error::ReferenceErrorCode::WriteToReadOnly,
                    multiname,
                    superclass.inner_class_definition(),
                ))
            } else {
                // In JIT mode in FP, setsuper on const slots and
                // getter-only accessors is silently ignored
                Ok(())
            }
        }
        None => Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::InvalidWrite,
            multiname,
            superclass.inner_class_definition(),
        )),
    }
}

/// Like `Value::call_method`, but uses the method specifically on the class's
/// instance vtable, not the instance vtable of the receiver. This is intended
/// to be used for supercalls.
fn call_method_super<'gc>(
    activation: &mut Activation<'_, 'gc>,
    superclass: ClassObject<'gc>,
    receiver: Value<'gc>,
    disp_id: u32,
    arguments: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let full_method = superclass
        .instance_vtable()
        .get_full_method(disp_id)
        .unwrap();

    // Only create callee if the method needs it
    let callee = if full_method.method.needs_arguments_object() {
        Some(FunctionObject::from_method(
            activation,
            full_method.method,
            full_method.scope(),
            Some(receiver),
            full_method.super_class_obj,
            Some(full_method.class),
        ))
    } else {
        None
    };

    exec(
        full_method.method,
        full_method.scope(),
        receiver,
        full_method.super_class_obj,
        Some(full_method.class),
        arguments,
        activation,
        callee,
    )
}
