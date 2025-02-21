use crate::avm2::class::Class;
pub use crate::avm2::globals::flash::utils::get_qualified_class_name;
use crate::avm2::metadata::Metadata;
use crate::avm2::method::Method;
use crate::avm2::object::{ArrayObject, ScriptObject, TObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::property::Property;
use crate::avm2::{Activation, Error, Multiname, Namespace, Object, Value};
use crate::string::{AvmString, StringContext};

use crate::avm2_stub_method;

use gc_arena::Gc;
use ruffle_macros::istr;

// Implements `avmplus.describeTypeJSON`
pub fn describe_type_json<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let flags = DescribeTypeFlags::from_bits(args.get_u32(activation, 1)?).expect("Invalid flags!");

    let value = args[0];
    let class_def = instance_class_describe_type(activation, value);
    let object = ScriptObject::new_object(activation);

    let mut used_class_def = class_def;
    if flags.contains(DescribeTypeFlags::USE_ITRAITS) {
        if let Some(i_class) = used_class_def.i_class() {
            used_class_def = i_class;
        } else {
            return Ok(Value::Null);
        }
    }

    let qualified_name = used_class_def
        .dollar_removed_name(activation.gc())
        .to_qualified_name(activation.gc());

    object.set_string_property_local(istr!("name"), qualified_name.into(), activation)?;

    object.set_string_property_local(
        istr!("isDynamic"),
        (!used_class_def.is_sealed()).into(),
        activation,
    )?;
    object.set_string_property_local(
        istr!("isFinal"),
        used_class_def.is_final().into(),
        activation,
    )?;
    object.set_string_property_local(
        istr!("isStatic"),
        value
            .as_object()
            .and_then(|o| o.as_class_object())
            .is_some()
            .into(),
        activation,
    )?;

    let traits = describe_internal_body(activation, used_class_def, flags)?;
    if flags.contains(DescribeTypeFlags::INCLUDE_TRAITS) {
        object.set_string_property_local(istr!("traits"), traits.into(), activation)?;
    } else {
        object.set_string_property_local(istr!("traits"), Value::Null, activation)?;
    }

    Ok(object.into())
}

bitflags::bitflags! {
    #[derive(Copy, Clone)]
    pub struct DescribeTypeFlags: u32 {
        const HIDE_NSURI_METHODS      = 1 << 0;
        const INCLUDE_BASES           = 1 << 1;
        const INCLUDE_INTERFACES      = 1 << 2;
        const INCLUDE_VARIABLES       = 1 << 3;
        const INCLUDE_ACCESSORS       = 1 << 4;
        const INCLUDE_METHODS         = 1 << 5;
        const INCLUDE_METADATA        = 1 << 6;
        const INCLUDE_CONSTRUCTOR     = 1 << 7;
        const INCLUDE_TRAITS          = 1 << 8;
        const USE_ITRAITS             = 1 << 9;
        const HIDE_OBJECT             = 1 << 10;
    }
}

fn describe_internal_body<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class_def: Class<'gc>,
    flags: DescribeTypeFlags,
) -> Result<Object<'gc>, Error<'gc>> {
    let mc = activation.gc();

    let traits = ScriptObject::new_object(activation);

    let bases = ArrayObject::empty(activation);
    let interfaces = ArrayObject::empty(activation);
    let variables = ArrayObject::empty(activation);
    let accessors = ArrayObject::empty(activation);
    let methods = ArrayObject::empty(activation);

    if flags.contains(DescribeTypeFlags::INCLUDE_BASES) {
        traits.set_string_property_local(istr!("bases"), bases.into(), activation)?;
    } else {
        traits.set_string_property_local(istr!("bases"), Value::Null, activation)?;
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_INTERFACES) {
        traits.set_string_property_local(istr!("interfaces"), interfaces.into(), activation)?;
    } else {
        traits.set_string_property_local(istr!("interfaces"), Value::Null, activation)?;
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_VARIABLES) {
        traits.set_string_property_local(istr!("variables"), variables.into(), activation)?;
    } else {
        traits.set_string_property_local(istr!("variables"), Value::Null, activation)?;
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_ACCESSORS) {
        traits.set_string_property_local(istr!("accessors"), accessors.into(), activation)?;
    } else {
        traits.set_string_property_local(istr!("accessors"), Value::Null, activation)?;
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_METHODS) {
        traits.set_string_property_local(istr!("methods"), methods.into(), activation)?;
    } else {
        traits.set_string_property_local(istr!("methods"), Value::Null, activation)?;
    }

    let mut bases_array = bases.array_storage_mut(mc);
    let mut interfaces_array = interfaces.array_storage_mut(mc);
    let mut variables_array = variables.array_storage_mut(mc);
    let mut accessors_array = accessors.array_storage_mut(mc);
    let mut methods_array = methods.array_storage_mut(mc);

    let superclass = class_def.super_class();

    if flags.contains(DescribeTypeFlags::INCLUDE_BASES) {
        let mut current_super_class = superclass;
        while let Some(super_class) = current_super_class {
            let super_name = super_class.name().to_qualified_name(mc);
            bases_array.push(super_name.into());
            current_super_class = super_class.super_class();
        }
    }

    let vtable = class_def.vtable();
    let super_vtable = class_def.super_class().map(|c| c.vtable());

    if flags.contains(DescribeTypeFlags::INCLUDE_INTERFACES) {
        for interface in &*class_def.all_interfaces() {
            let interface_name = interface.name().to_qualified_name(mc);
            interfaces_array.push(interface_name.into());
        }
    }

    // Implement the weird 'HIDE_NSURI_METHODS' behavior from avmplus:
    // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/TypeDescriber.cpp#L237
    let mut skip_ns: Vec<Namespace<'_>> = Vec::new();
    if let Some(super_vtable) = super_vtable {
        for (_, ns, prop) in super_vtable.resolved_traits().iter() {
            if !ns.as_uri(activation.strings()).is_empty() {
                if let Property::Method { .. } = prop {
                    if !skip_ns
                        .iter()
                        .any(|other_ns| other_ns.exact_version_match(ns))
                    {
                        skip_ns.push(ns);
                    }
                }
            }
        }
    }

    // FIXME - avmplus iterates over their own hashtable, so the order in the final XML
    // is different
    for (prop_name, ns, prop) in vtable.resolved_traits().iter() {
        if !ns.is_public_ignoring_ns() {
            continue;
        }

        if !ns.matches_api_version(activation.avm2().root_api_version) {
            continue;
        }

        if flags.contains(DescribeTypeFlags::HIDE_NSURI_METHODS)
            && skip_ns
                .iter()
                .any(|other_ns| ns.exact_version_match(*other_ns))
        {
            continue;
        }

        let uri = ns.as_uri_opt().filter(|uri| !uri.is_empty());

        match prop {
            Property::ConstSlot { slot_id } | Property::Slot { slot_id } => {
                if !flags.contains(DescribeTypeFlags::INCLUDE_VARIABLES) {
                    continue;
                }
                let prop_class_name = vtable.slot_class_name(activation.strings(), *slot_id)?;

                let access = match prop {
                    Property::ConstSlot { .. } => istr!("readonly"),
                    Property::Slot { .. } => istr!("readwrite"),
                    _ => unreachable!(),
                };

                let trait_metadata = vtable.get_metadata_for_slot(slot_id);

                let variable = ScriptObject::new_object(activation);
                variable.set_string_property_local(istr!("name"), prop_name.into(), activation)?;
                variable.set_string_property_local(
                    istr!("type"),
                    prop_class_name.into(),
                    activation,
                )?;
                variable.set_string_property_local(istr!("access"), access.into(), activation)?;
                variable.set_string_property_local(
                    istr!("uri"),
                    uri.map_or(Value::Null, |u| u.into()),
                    activation,
                )?;

                variable.set_string_property_local(istr!("metadata"), Value::Null, activation)?;

                if flags.contains(DescribeTypeFlags::INCLUDE_METADATA) {
                    let metadata_object = ArrayObject::empty(activation);
                    if let Some(metadata) = trait_metadata {
                        write_metadata(metadata_object, &metadata, activation)?;
                    }
                    variable.set_string_property_local(
                        istr!("metadata"),
                        metadata_object.into(),
                        activation,
                    )?;
                }

                variables_array.push(variable.into());
            }
            Property::Method { disp_id } => {
                if !flags.contains(DescribeTypeFlags::INCLUDE_METHODS) {
                    continue;
                }
                let method = vtable
                    .get_full_method(*disp_id)
                    .unwrap_or_else(|| panic!("Missing method for id {disp_id:?}"));

                // Don't include methods that also exist in any interface
                if method
                    .class
                    .all_interfaces()
                    .iter()
                    .any(|interface| interface.vtable().has_trait(&Multiname::new(ns, prop_name)))
                {
                    continue;
                }

                let return_type_name =
                    display_name(activation.strings(), method.method.return_type());
                let declared_by = method.class;

                if flags.contains(DescribeTypeFlags::HIDE_OBJECT)
                    && declared_by == activation.avm2().class_defs().object
                {
                    continue;
                }

                let declared_by_name = declared_by.dollar_removed_name(mc).to_qualified_name(mc);

                let trait_metadata = vtable.get_metadata_for_disp(disp_id);

                let method_obj = ScriptObject::new_object(activation);

                method_obj.set_string_property_local(
                    istr!("name"),
                    prop_name.into(),
                    activation,
                )?;
                method_obj.set_string_property_local(
                    istr!("returnType"),
                    return_type_name.into(),
                    activation,
                )?;
                method_obj.set_string_property_local(
                    istr!("declaredBy"),
                    declared_by_name.into(),
                    activation,
                )?;

                method_obj.set_string_property_local(
                    istr!("uri"),
                    uri.map_or(Value::Null, |u| u.into()),
                    activation,
                )?;

                let params = write_params(&method.method, activation)?;
                method_obj.set_string_property_local(
                    istr!("parameters"),
                    params.into(),
                    activation,
                )?;

                method_obj.set_string_property_local(istr!("metadata"), Value::Null, activation)?;

                if flags.contains(DescribeTypeFlags::INCLUDE_METADATA) {
                    let metadata_object = ArrayObject::empty(activation);
                    if let Some(metadata) = trait_metadata {
                        write_metadata(metadata_object, &metadata, activation)?;
                    }
                    method_obj.set_string_property_local(
                        istr!("metadata"),
                        metadata_object.into(),
                        activation,
                    )?;
                }
                methods_array.push(method_obj.into());
            }
            Property::Virtual { get, set } => {
                if !flags.contains(DescribeTypeFlags::INCLUDE_ACCESSORS) {
                    continue;
                }
                let access = match (get, set) {
                    (Some(_), Some(_)) => istr!("readwrite"),
                    (Some(_), None) => istr!("readonly"),
                    (None, Some(_)) => istr!("writeonly"),
                    (None, None) => unreachable!(),
                };

                // For getters, obtain the type by looking at the getter return type.
                // For setters, obtain the type by looking at the setter's first parameter.
                let (method_type, defining_class) = if let Some(get) = get {
                    let getter = vtable
                        .get_full_method(*get)
                        .unwrap_or_else(|| panic!("Missing 'get' method for id {get:?}"));
                    (getter.method.return_type(), getter.class)
                } else if let Some(set) = set {
                    let setter = vtable
                        .get_full_method(*set)
                        .unwrap_or_else(|| panic!("Missing 'set' method for id {set:?}"));
                    (setter.method.signature()[0].param_type_name, setter.class)
                } else {
                    unreachable!();
                };

                // Don't include virtual properties that also exist in any interface
                if defining_class
                    .all_interfaces()
                    .iter()
                    .any(|interface| interface.vtable().has_trait(&Multiname::new(ns, prop_name)))
                {
                    continue;
                }

                let uri = ns.as_uri_opt().filter(|uri| !uri.is_empty());
                let accessor_type = display_name(activation.strings(), method_type);
                let declared_by = defining_class.dollar_removed_name(mc).to_qualified_name(mc);

                let accessor_obj = ScriptObject::new_object(activation);
                accessor_obj.set_string_property_local(
                    istr!("name"),
                    prop_name.into(),
                    activation,
                )?;
                accessor_obj.set_string_property_local(
                    istr!("access"),
                    access.into(),
                    activation,
                )?;
                accessor_obj.set_string_property_local(
                    istr!("type"),
                    accessor_type.into(),
                    activation,
                )?;
                accessor_obj.set_string_property_local(
                    istr!("declaredBy"),
                    declared_by.into(),
                    activation,
                )?;
                accessor_obj.set_string_property_local(
                    istr!("uri"),
                    uri.map_or(Value::Null, |u| u.into()),
                    activation,
                )?;

                let metadata_object = ArrayObject::empty(activation);

                if let Some(get_disp_id) = get {
                    if let Some(metadata) = vtable.get_metadata_for_disp(get_disp_id) {
                        write_metadata(metadata_object, &metadata, activation)?;
                    }
                }

                if let Some(set_disp_id) = set {
                    if let Some(metadata) = vtable.get_metadata_for_disp(set_disp_id) {
                        write_metadata(metadata_object, &metadata, activation)?;
                    }
                }

                if flags.contains(DescribeTypeFlags::INCLUDE_METADATA)
                    && metadata_object.array_storage().length() > 0
                {
                    accessor_obj.set_string_property_local(
                        istr!("metadata"),
                        metadata_object.into(),
                        activation,
                    )?;
                } else {
                    accessor_obj.set_string_property_local(
                        istr!("metadata"),
                        Value::Null,
                        activation,
                    )?;
                }

                accessors_array.push(accessor_obj.into());
            }
        }
    }

    let constructor = class_def.instance_init();
    // Flash only shows a <constructor> element if it has at least one parameter
    if flags.contains(DescribeTypeFlags::INCLUDE_CONSTRUCTOR) && !constructor.signature().is_empty()
    {
        let params = write_params(&constructor, activation)?;
        traits.set_string_property_local(istr!("constructor"), params.into(), activation)?;
    } else {
        // This is needed to override the normal 'constructor' property
        traits.set_string_property_local(istr!("constructor"), Value::Null, activation)?;
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_METADATA) {
        avm2_stub_method!(
            activation,
            "avmplus",
            "describeTypeJSON",
            "with top-level metadata"
        );

        let metadata_object = ArrayObject::empty(activation);
        traits.set_string_property_local(istr!("metadata"), metadata_object.into(), activation)?;
    } else {
        traits.set_string_property_local(istr!("metadata"), Value::Null, activation)?;
    }

    Ok(traits)
}

fn display_name<'gc>(
    context: &mut StringContext<'gc>,
    name: Option<Gc<'gc, Multiname<'gc>>>,
) -> AvmString<'gc> {
    if let Some(name) = name {
        name.to_qualified_name_or_star(context)
    } else {
        istr!(context, "*")
    }
}

fn write_params<'gc>(
    method: &Method<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<ArrayObject<'gc>, Error<'gc>> {
    let params = ArrayObject::empty(activation);
    let mut params_array = params.array_storage_mut(activation.gc());
    for param in method.signature() {
        let param_type_name = display_name(activation.strings(), param.param_type_name);
        let optional = param.default_value.is_some();
        let param_obj = ScriptObject::new_object(activation);
        param_obj.set_string_property_local(istr!("type"), param_type_name.into(), activation)?;
        param_obj.set_string_property_local(istr!("optional"), optional.into(), activation)?;
        params_array.push(param_obj.into());
    }
    Ok(params)
}

fn write_metadata<'gc>(
    metadata_object: ArrayObject<'gc>,
    trait_metadata: &[Metadata<'gc>],
    activation: &mut Activation<'_, 'gc>,
) -> Result<(), Error<'gc>> {
    let mut metadata_array = metadata_object.array_storage_mut(activation.gc());

    for single_trait in trait_metadata.iter() {
        metadata_array.push(single_trait.as_json_object(activation)?.into());
    }
    Ok(())
}

/// Like `Value::instance_class`, but supports Value::Null and Value::Undefined,
/// and returns `int` for Value::Integer instead of `Number`.
///
/// Used for `describeType`, `getQualifiedClassName`, and `getQualifiedSuperClassName`.
pub fn instance_class_describe_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
) -> Class<'gc> {
    let class_defs = activation.avm2().class_defs();

    match value {
        Value::Null => class_defs.null,
        Value::Undefined => class_defs.void,
        Value::Integer(_) => class_defs.int,
        _ => value.instance_class(activation),
    }
}
