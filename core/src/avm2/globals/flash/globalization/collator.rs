//! `flash.globalization.Collator` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_1508;
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::string::locale_compare;
use crate::avm2::object::VectorObject;
pub use crate::avm2::object::collator_allocator;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::string::AvmString;
use crate::{avm2_stub_constructor, avm2_stub_getter, avm2_stub_method, avm2_stub_setter};

pub fn ctor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // Note that this behavior is platform-dependent. Linux for instance rejects
    // unknown locales (including i-default) and throws
    //
    //   TypeError: Error #2007: Parameter Constructor Failed must be non-null.
    //
    // In addition, different platforms normalize locale IDs differently.
    // The current implementation doesn't do any normalization.

    avm2_stub_constructor!(
        activation,
        "flash.globalization.Collator",
        "platform-dependent behavior"
    );

    let this = this.as_object().unwrap();
    let collator = this.as_collator().expect("Must be Collator object");

    let requested_locale_id_name =
        args.get_string_non_null(activation, 0, "requestedLocaleIDName")?;
    let initial_mode = args.get_string_non_null(activation, 1, "initialMode")?;

    if &initial_mode != b"sorting" && &initial_mode != b"matching" {
        return Err(make_error_1508(activation, "initialMode"));
    }

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

    Ok(collator.requested_locale_id_name().into())
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
    // The output is highly platform-dependent, but there are some
    // characteristics of the output:
    //
    //  1. It can contain duplicate entries, but it's not common.
    //     Noticed one on Windows: "zh-MO@collation=stroke".
    //
    //  2. It appears to be *mostly* sorted lexicographically, but not always.
    //     On Linux "eo" and "tok" were observed right before "be-Latn-BY", the
    //     rest was sorted.
    //
    //  3. It appears to use the BCP-47 format with glibc modifiers.
    //
    //  4. Both 2- and 3-letter locales are present.
    //
    //  5. Example observed script subtags include: az-Cyrl-AZ, bm-Latn,
    //     ccp-Cakm-BD, chr-Cher-US, jv-Java-ID, ks-Arab-IN, ks-Deva-IN,
    //     zh-Hans-HK, shi-Tfng.
    //
    //  6. Region subtags can contain numbers, such as: ar-001, en-001, en-029,
    //     en-150, eo-001, es-419.
    //
    //  7. There are IDs that are prefixes of others, such as: bm, bm-Latn,
    //     bm-Latn-ML, hu-HU, hu-HU@collation=technical.
    //
    //  8. Some IDs contain additional subtags, such as: ca-ES-VALENCIA.
    //
    //  9. Private-use tags can be present, such as: x-iv_mathan.
    //
    //  10. Some IDs end with a glibc modifier, such as (Windows):
    //      de-DE@collation=phonebook, es-ES@collation=traditional,
    //      hu-HU@collation=technical, ja-JP@collation=stroke,
    //      ka-GE@collation=modern, zh-CN@collation=phonebook,
    //      zh-TW@collation=pinyin;
    //      and (Linux): ga-IE@currency=EUR, eu-ES@currency=EUR,
    //      gez-ET@collation=abegede.

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
