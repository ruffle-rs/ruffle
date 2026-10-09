//! `flash.globalization.Collator` native methods

use crate::avm2::activation::Activation;
use crate::avm2::error::make_error_1508;
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::string::locale_compare;
pub use crate::avm2::object::collator_allocator;
use crate::avm2::object::{CollatorOptions, LastOperationStatus, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Avm2StrRepresentable, Error};
use crate::string::AvmString;
use crate::{avm2_stub_constructor, avm2_stub_getter, avm2_stub_method};

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
    let collator = this.as_collator().unwrap();

    let requested_locale_id_name =
        args.get_string_non_null(activation, 0, "requestedLocaleIDName")?;
    let initial_mode = args.get_string_non_null(activation, 1, "initialMode")?;

    if &initial_mode != b"sorting" && &initial_mode != b"matching" {
        return Err(make_error_1508(activation, "initialMode"));
    }

    collator.set_requested_locale_id_name(requested_locale_id_name, activation.gc());
    if &initial_mode == b"matching" {
        collator.set_options(CollatorOptions::matching());
    }

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
    let collator = this.as_collator().unwrap();

    Ok(collator.requested_locale_id_name().into())
}

pub fn get_ignore_case<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .options()
        .contains(CollatorOptions::IGNORE_CASE)
        .into())
}

pub fn set_ignore_case<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    collator.set_option(CollatorOptions::IGNORE_CASE, args.get_bool(0));
    // TODO: This should set NoError when we start supporting it.
    collator.set_last_operation_status(LastOperationStatus::UnsupportedError);

    Ok(Value::Undefined)
}

pub fn get_ignore_character_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .options()
        .contains(CollatorOptions::IGNORE_CHARACTER_WIDTH)
        .into())
}

pub fn set_ignore_character_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    collator.set_option(CollatorOptions::IGNORE_CHARACTER_WIDTH, args.get_bool(0));
    // TODO: This should set NoError when we start supporting it.
    // It is supported on Windows, unsupported on Linux.
    collator.set_last_operation_status(LastOperationStatus::UnsupportedError);

    Ok(Value::Undefined)
}

pub fn get_ignore_diacritics<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .options()
        .contains(CollatorOptions::IGNORE_DIACRITICS)
        .into())
}

pub fn set_ignore_diacritics<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    collator.set_option(CollatorOptions::IGNORE_DIACRITICS, args.get_bool(0));
    // TODO: This should set NoError when we start supporting it.
    // It is supported on Windows, unsupported on Linux.
    collator.set_last_operation_status(LastOperationStatus::UnsupportedError);

    Ok(Value::Undefined)
}

pub fn get_ignore_kana_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .options()
        .contains(CollatorOptions::IGNORE_KANA_TYPE)
        .into())
}

pub fn set_ignore_kana_type<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    collator.set_option(CollatorOptions::IGNORE_KANA_TYPE, args.get_bool(0));
    // TODO: This should set NoError when we start supporting it.
    // It is supported on Windows, unsupported on Linux.
    collator.set_last_operation_status(LastOperationStatus::UnsupportedError);

    Ok(Value::Undefined)
}

pub fn get_ignore_symbols<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .options()
        .contains(CollatorOptions::IGNORE_SYMBOLS)
        .into())
}

pub fn set_ignore_symbols<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    collator.set_option(CollatorOptions::IGNORE_SYMBOLS, args.get_bool(0));
    // TODO: This should set NoError when we start supporting it.
    // It is supported on Windows, unsupported on Linux.
    collator.set_last_operation_status(LastOperationStatus::UnsupportedError);

    Ok(Value::Undefined)
}

pub fn get_last_operation_status<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .last_operation_status()
        .as_avm2_str(activation)
        .into())
}

pub fn get_numeric_comparison<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    Ok(collator
        .options()
        .contains(CollatorOptions::NUMERIC_COMPARISON)
        .into())
}

pub fn set_numeric_comparison<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let collator = this.as_collator().unwrap();

    collator.set_option(CollatorOptions::NUMERIC_COMPARISON, args.get_bool(0));
    // This is unsupported on both Windows and Linux.
    collator.set_last_operation_status(LastOperationStatus::UnsupportedError);

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
