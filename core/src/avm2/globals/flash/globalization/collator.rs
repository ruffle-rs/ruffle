//! `flash.globalization.Collator` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::string::locale_compare;
use crate::avm2::object::VectorObject;
pub use crate::avm2::object::collator_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::string::AvmString;
use crate::{avm2_stub_constructor, avm2_stub_getter, avm2_stub_method, avm2_stub_setter};

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_constructor!(activation, "flash.globalization.Collator");

    let this = this.as_object().unwrap();
    let collator = this.as_collator().expect("Must be Collator object");

    let requested_locale_id_name = args.try_get_string(0);
    collator.set_requested_locale_id_name(requested_locale_id_name, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_actual_locale_id_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.globalization.Collator",
        "actualLocaleIDName"
    );

    Ok(AvmString::new_utf8(activation.gc(), "en-US").into())
}

pub fn get_requested_locale_id_name<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().expect("Must be Collator object");

    Ok(collator
        .requested_locale_id_name()
        .map_or(Value::Null, Value::from))
}

pub fn get_ignore_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.globalization.Collator", "ignoreCase");

    Ok(false.into())
}

pub fn set_ignore_case<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.globalization.Collator", "ignoreCase");

    Ok(Value::Undefined)
}

pub fn get_ignore_character_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.globalization.Collator",
        "ignoreCharacterWidth"
    );

    Ok(false.into())
}

pub fn set_ignore_character_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(
        activation,
        "flash.globalization.Collator",
        "ignoreCharacterWidth"
    );

    Ok(Value::Undefined)
}

pub fn get_ignore_diacritics<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.globalization.Collator",
        "ignoreDiacritics"
    );

    Ok(false.into())
}

pub fn set_ignore_diacritics<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(
        activation,
        "flash.globalization.Collator",
        "ignoreDiacritics"
    );

    Ok(Value::Undefined)
}

pub fn get_ignore_kana_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.globalization.Collator", "ignoreKanaType");

    Ok(false.into())
}

pub fn set_ignore_kana_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.globalization.Collator", "ignoreKanaType");

    Ok(Value::Undefined)
}

pub fn get_ignore_symbols<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(activation, "flash.globalization.Collator", "ignoreSymbols");

    Ok(false.into())
}

pub fn set_ignore_symbols<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(activation, "flash.globalization.Collator", "ignoreSymbols");

    Ok(Value::Undefined)
}

pub fn get_last_operation_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.globalization.Collator",
        "lastOperationStatus"
    );

    Ok(AvmString::new_utf8(activation.gc(), "noError").into())
}

pub fn get_numeric_comparison<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.globalization.Collator",
        "numericComparison"
    );

    Ok(false.into())
}

pub fn set_numeric_comparison<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_setter!(
        activation,
        "flash.globalization.Collator",
        "numericComparison"
    );

    Ok(Value::Undefined)
}

pub fn compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.globalization.Collator", "compare");

    let string1 = args.get_value(0);
    let string2 = args.get_value(1);

    locale_compare(activation, string1, FunctionArgs::from_slice(&[string2]))
}

pub fn equals<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.globalization.Collator", "equals");

    let string1 = args.get_value(0);
    let string2 = args.get_value(1);

    let result = locale_compare(activation, string1, FunctionArgs::from_slice(&[string2]))?;
    Ok((result.coerce_to_i32(activation)? == 0).into())
}

pub fn get_available_locale_id_names<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(
        activation,
        "flash.globalization.Collator",
        "getAvailableLocaleIDNames"
    );

    let storage = VectorStorage::from_values(
        vec![AvmString::new_utf8(activation.gc(), "en-US").into()],
        false,
        Some(activation.avm2().class_defs().string),
    );
    Ok(VectorObject::from_vector(storage, activation).into())
}
