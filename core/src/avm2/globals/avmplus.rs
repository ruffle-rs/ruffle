use crate::avm2::class::Class;
pub use crate::avm2::globals::flash::utils::get_qualified_class_name;
use crate::avm2::metadata::Metadata;
use crate::avm2::method::Method;
use crate::avm2::object::{ArrayObject, ScriptObject, TObject as _};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::property::Property;
use crate::avm2::{Activation, Error, Multiname, Namespace, Object, Value};
use crate::context::UpdateContext;
use crate::string::{AvmString, StringContext};

use crate::avm2_stub_method_context;

use gc_arena::Gc;
use ruffle_macros::istr;

// Implements `avmplus.describeTypeJSON`
pub fn describe_type_json<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let flags = DescribeTypeFlags::from_bits(args.get_u32(1)).expect("Invalid flags!");

    let value = args.get_value(0);
    let class_def = instance_class_describe_type(activation, value);
    let object = ScriptObject::new_object(activation.context);

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

    object.set_dynamic_property(istr!("name"), qualified_name.into(), activation.gc());

    object.set_dynamic_property(
        istr!("isDynamic"),
        (!used_class_def.is_sealed()).into(),
        activation.gc(),
    );
    object.set_dynamic_property(
        istr!("isFinal"),
        used_class_def.is_final().into(),
        activation.gc(),
    );
    object.set_dynamic_property(
        istr!("isStatic"),
        value
            .as_object()
            .and_then(|o| o.as_class_object())
            .is_some()
            .into(),
        activation.gc(),
    );

    let traits = describe_internal_body(activation.context, used_class_def, flags);
    if flags.contains(DescribeTypeFlags::INCLUDE_TRAITS) {
        object.set_dynamic_property(istr!("traits"), traits.into(), activation.gc());
    } else {
        object.set_dynamic_property(istr!("traits"), Value::Null, activation.gc());
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
    context: &mut UpdateContext<'gc>,
    class_def: Class<'gc>,
    flags: DescribeTypeFlags,
) -> Object<'gc> {
    let mc = context.gc();

    let traits = ScriptObject::new_object(context);

    let bases = ArrayObject::empty(context);
    let interfaces = ArrayObject::empty(context);
    let variables = ArrayObject::empty(context);
    let accessors = ArrayObject::empty(context);
    let methods = ArrayObject::empty(context);

    if flags.contains(DescribeTypeFlags::INCLUDE_BASES) {
        traits.set_dynamic_property(istr!(context, "bases"), bases.into(), mc);
    } else {
        traits.set_dynamic_property(istr!(context, "bases"), Value::Null, mc);
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_INTERFACES) {
        traits.set_dynamic_property(istr!(context, "interfaces"), interfaces.into(), mc);
    } else {
        traits.set_dynamic_property(istr!(context, "interfaces"), Value::Null, mc);
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_VARIABLES) {
        traits.set_dynamic_property(istr!(context, "variables"), variables.into(), mc);
    } else {
        traits.set_dynamic_property(istr!(context, "variables"), Value::Null, mc);
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_ACCESSORS) {
        traits.set_dynamic_property(istr!(context, "accessors"), accessors.into(), mc);
    } else {
        traits.set_dynamic_property(istr!(context, "accessors"), Value::Null, mc);
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_METHODS) {
        traits.set_dynamic_property(istr!(context, "methods"), methods.into(), mc);
    } else {
        traits.set_dynamic_property(istr!(context, "methods"), Value::Null, mc);
    }

    let mut bases_array = bases.storage_mut(mc);
    let mut interfaces_array = interfaces.storage_mut(mc);
    let mut variables_array = variables.storage_mut(mc);
    let mut accessors_array = accessors.storage_mut(mc);
    let mut methods_array = methods.storage_mut(mc);

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
        for interface in class_def.all_interfaces() {
            let interface_name = interface.name().to_qualified_name(mc);
            interfaces_array.push(interface_name.into());
        }
    }

    // Implement the weird 'HIDE_NSURI_METHODS' behavior from avmplus:
    // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/TypeDescriber.cpp#L237
    let mut skip_ns: Vec<Namespace<'_>> = Vec::new();
    if let Some(super_vtable) = super_vtable {
        for (_, ns, prop) in super_vtable.resolved_traits().iter() {
            if !ns.as_uri(&mut context.strings).is_empty() {
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

        if !ns.matches_api_version(context.avm2.root_api_version) {
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
                let prop_class_name = vtable.slot_class_name(&mut context.strings, *slot_id);

                let access = match prop {
                    Property::ConstSlot { .. } => istr!(context, "readonly"),
                    Property::Slot { .. } => istr!(context, "readwrite"),
                    _ => unreachable!(),
                };

                let trait_metadata = vtable.get_metadata_for_slot(*slot_id);

                let variable = ScriptObject::new_object(context);
                variable.set_dynamic_property(istr!(context, "name"), prop_name.into(), mc);
                variable.set_dynamic_property(istr!(context, "type"), prop_class_name.into(), mc);
                variable.set_dynamic_property(istr!(context, "access"), access.into(), mc);
                variable.set_dynamic_property(
                    istr!(context, "uri"),
                    uri.map_or(Value::Null, |u| u.into()),
                    mc,
                );

                variable.set_dynamic_property(istr!(context, "metadata"), Value::Null, mc);

                if flags.contains(DescribeTypeFlags::INCLUDE_METADATA) {
                    let metadata_object = ArrayObject::empty(context);
                    if let Some(metadata) = trait_metadata {
                        write_metadata(metadata_object, metadata, context);
                    }
                    variable.set_dynamic_property(
                        istr!(context, "metadata"),
                        metadata_object.into(),
                        mc,
                    );
                }

                variables_array.push(variable.into());
            }
            Property::Method { disp_id } => {
                if !flags.contains(DescribeTypeFlags::INCLUDE_METHODS) {
                    continue;
                }

                let method = vtable
                    .get_method(*disp_id)
                    .unwrap_or_else(|| panic!("Missing method for id {disp_id:?}"));
                let declared_by = method
                    .bound_class()
                    .expect("Method on vtable is classbound");

                // Don't include methods that also exist in any interface
                if declared_by
                    .all_interfaces()
                    .iter()
                    .any(|interface| interface.vtable().has_trait(&Multiname::new(ns, prop_name)))
                {
                    continue;
                }

                let return_type_name = display_name(&mut context.strings, method.return_type());

                if flags.contains(DescribeTypeFlags::HIDE_OBJECT)
                    && declared_by == context.avm2.class_defs().object
                {
                    continue;
                }

                let declared_by_name = declared_by.dollar_removed_name(mc).to_qualified_name(mc);

                let trait_metadata = vtable.get_metadata_for_disp(*disp_id);

                let method_obj = ScriptObject::new_object(context);

                method_obj.set_dynamic_property(istr!(context, "name"), prop_name.into(), mc);
                method_obj.set_dynamic_property(
                    istr!(context, "returnType"),
                    return_type_name.into(),
                    mc,
                );
                method_obj.set_dynamic_property(
                    istr!(context, "declaredBy"),
                    declared_by_name.into(),
                    mc,
                );

                method_obj.set_dynamic_property(
                    istr!(context, "uri"),
                    uri.map_or(Value::Null, |u| u.into()),
                    mc,
                );

                let params = write_params(method, context);
                method_obj.set_dynamic_property(istr!(context, "parameters"), params.into(), mc);

                method_obj.set_dynamic_property(istr!(context, "metadata"), Value::Null, mc);

                if flags.contains(DescribeTypeFlags::INCLUDE_METADATA) {
                    let metadata_object = ArrayObject::empty(context);
                    if let Some(metadata) = trait_metadata {
                        write_metadata(metadata_object, metadata, context);
                    }
                    method_obj.set_dynamic_property(
                        istr!(context, "metadata"),
                        metadata_object.into(),
                        mc,
                    );
                }
                methods_array.push(method_obj.into());
            }
            Property::Virtual { get, set } => {
                if !flags.contains(DescribeTypeFlags::INCLUDE_ACCESSORS) {
                    continue;
                }
                let access = match (get, set) {
                    (Some(_), Some(_)) => istr!(context, "readwrite"),
                    (Some(_), None) => istr!(context, "readonly"),
                    (None, Some(_)) => istr!(context, "writeonly"),
                    (None, None) => unreachable!(),
                };

                // For getters, obtain the type by looking at the getter return type.
                // For setters, obtain the type by looking at the setter's first parameter.
                let (method_type, defining_class) = if let Some(get) = get {
                    let get_method = vtable
                        .get_method(*get)
                        .unwrap_or_else(|| panic!("Missing 'get' method for id {get:?}"));
                    let bound_class = get_method
                        .bound_class()
                        .expect("Method on vtable is classbound");

                    (get_method.return_type(), bound_class)
                } else if let Some(set) = set {
                    let set_method = vtable
                        .get_method(*set)
                        .unwrap_or_else(|| panic!("Missing 'set' method for id {set:?}"));
                    let bound_class = set_method
                        .bound_class()
                        .expect("Method on vtable is classbound");

                    (set_method.signature()[0].param_type_name, bound_class)
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
                let accessor_type = display_name(&mut context.strings, method_type);
                let declared_by = defining_class.dollar_removed_name(mc).to_qualified_name(mc);

                let accessor_obj = ScriptObject::new_object(context);
                accessor_obj.set_dynamic_property(istr!(context, "name"), prop_name.into(), mc);
                accessor_obj.set_dynamic_property(istr!(context, "access"), access.into(), mc);
                accessor_obj.set_dynamic_property(istr!(context, "type"), accessor_type.into(), mc);
                accessor_obj.set_dynamic_property(
                    istr!(context, "declaredBy"),
                    declared_by.into(),
                    mc,
                );
                accessor_obj.set_dynamic_property(
                    istr!(context, "uri"),
                    uri.map_or(Value::Null, |u| u.into()),
                    mc,
                );

                let metadata_object = ArrayObject::empty(context);

                if let Some(get_disp_id) = get {
                    if let Some(metadata) = vtable.get_metadata_for_disp(*get_disp_id) {
                        write_metadata(metadata_object, metadata, context);
                    }
                }

                if let Some(set_disp_id) = set {
                    if let Some(metadata) = vtable.get_metadata_for_disp(*set_disp_id) {
                        write_metadata(metadata_object, metadata, context);
                    }
                }

                if flags.contains(DescribeTypeFlags::INCLUDE_METADATA)
                    && metadata_object.storage().length() > 0
                {
                    accessor_obj.set_dynamic_property(
                        istr!(context, "metadata"),
                        metadata_object.into(),
                        mc,
                    );
                } else {
                    accessor_obj.set_dynamic_property(istr!(context, "metadata"), Value::Null, mc);
                }

                accessors_array.push(accessor_obj.into());
            }
        }
    }

    let constructor = class_def.instance_init();
    // Flash only shows a <constructor> element if it has at least one parameter
    if let Some(constructor) = constructor.filter(|c| {
        !c.signature().is_empty() && flags.contains(DescribeTypeFlags::INCLUDE_CONSTRUCTOR)
    }) {
        let params = write_params(constructor, context);
        traits.set_dynamic_property(istr!(context, "constructor"), params.into(), mc);
    } else {
        // This is needed to override the normal 'constructor' property
        traits.set_dynamic_property(istr!(context, "constructor"), Value::Null, mc);
    }

    if flags.contains(DescribeTypeFlags::INCLUDE_METADATA) {
        avm2_stub_method_context!(
            context,
            "avmplus",
            "describeTypeJSON",
            "with top-level metadata"
        );

        let metadata_object = ArrayObject::empty(context);
        traits.set_dynamic_property(istr!(context, "metadata"), metadata_object.into(), mc);
    } else {
        traits.set_dynamic_property(istr!(context, "metadata"), Value::Null, mc);
    }

    traits
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

fn write_params<'gc>(method: Method<'gc>, context: &mut UpdateContext<'gc>) -> ArrayObject<'gc> {
    let mc = context.gc();

    let params = ArrayObject::empty(context);
    let mut params_array = params.storage_mut(mc);
    for param in method.signature() {
        let param_type_name = display_name(&mut context.strings, param.param_type_name);
        let optional = param.default_value.is_some();
        let param_obj = ScriptObject::new_object(context);
        param_obj.set_dynamic_property(istr!(context, "type"), param_type_name.into(), mc);
        param_obj.set_dynamic_property(istr!(context, "optional"), optional.into(), mc);
        params_array.push(param_obj.into());
    }
    params
}

fn write_metadata<'gc>(
    metadata_object: ArrayObject<'gc>,
    trait_metadata: &[Metadata<'gc>],
    context: &mut UpdateContext<'gc>,
) {
    let mut metadata_array = metadata_object.storage_mut(context.gc());

    for single_trait in trait_metadata.iter() {
        metadata_array.push(single_trait.as_json_object(context).into());
    }
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

    match value.normalize() {
        Value::Null => class_defs.null,
        Value::Undefined => class_defs.void,
        Value::Integer(_) => class_defs.int,
        value => value.instance_class(activation),
    }
}
