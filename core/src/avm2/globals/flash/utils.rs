//! `flash.utils` namespace

use crate::avm2::metadata::Metadata;
use crate::avm2::method::Method;
use crate::avm2::object::TObject;
use crate::avm2::property::Property;
use crate::avm2::ClassObject;
use crate::avm2::{Activation, Error, Object, Value};
use crate::string::AvmString;
use crate::string::WString;
use instant::Instant;
use std::fmt::Write;

pub mod byte_array;
pub mod dictionary;
pub mod proxy;
pub mod timer;

/// Implements `flash.utils.getTimer`
pub fn get_timer<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok((Instant::now()
        .duration_since(activation.context.start_time)
        .as_millis() as u32)
        .into())
}

/// Implements `flash.utils.setInterval`
pub fn set_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Err(Error::from("setInterval: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setInterval: not enough arguments")
            .as_object()
            .ok_or("setInterval: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setInterval: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        false,
    )))
}

/// Implements `flash.utils.clearInterval`
pub fn clear_interval<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .ok_or("clearInterval: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.setTimeout`
pub fn set_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Err(Error::from("setTimeout: not enough arguments"));
    }
    let (args, params) = args.split_at(2);
    let callback = crate::timer::TimerCallback::Avm2Callback {
        closure: args
            .get(0)
            .expect("setTimeout: not enough arguments")
            .as_object()
            .ok_or("setTimeout: argument 0 is not an object")?,
        params: params.to_vec(),
    };
    let interval = args
        .get(1)
        .expect("setTimeout: not enough arguments")
        .coerce_to_number(activation)?;
    Ok(Value::Integer(activation.context.timers.add_timer(
        callback,
        interval as i32,
        true,
    )))
}

/// Implements `flash.utils.clearTimeout`
pub fn clear_timeout<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let id = args
        .get(0)
        .ok_or("clearTimeout: not enough arguments")?
        .coerce_to_number(activation)?;
    activation.context.timers.remove(id as i32);
    Ok(Value::Undefined)
}

/// Implements `flash.utils.escapeMultiByte`
pub fn escape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let utf8 = s.as_wstr().to_utf8_lossy();
    let mut result = WString::new();
    for byte in utf8.as_bytes() {
        if *byte == 0 {
            break;
        }
        if byte.is_ascii_alphanumeric() {
            result.push_byte(*byte);
        } else {
            let _ = write!(&mut result, "%{byte:02X}");
        }
    }
    Ok(AvmString::new(activation.context.gc_context, result).into())
}

fn handle_percent<I>(chars: &mut I) -> Option<u8>
where
    I: Iterator<Item = char>,
{
    let high = chars.next()?.to_digit(16)? as u8;
    let low = chars.next()?.to_digit(16)? as u8;
    Some(low | (high << 4))
}

/// Implements `flash.utils.unescapeMultiByte`
pub fn unescape_multi_byte<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let s = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    let bs = s.as_wstr();
    let mut buf = WString::new();
    let chars = bs.chars().map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER));

    let mut chars = chars.peekable();
    let mut utf8_bytes = Vec::new();
    while let Some(c) = chars.next() {
        if c == '\0' {
            break;
        }
        if c == '%' {
            while let Some(b) = handle_percent(&mut chars) {
                utf8_bytes.push(b);
                if !matches!(chars.peek(), Some('%')) {
                    break;
                }
                chars.next();
            }
            buf.push_utf8_bytes(&utf8_bytes);
            utf8_bytes.clear();
            continue;
        }

        buf.push_char(c);
    }
    let v = AvmString::new(activation.context.gc_context, buf);
    Ok(v.into())
}

/// Implements `flash.utils.getQualifiedClassName`
pub fn get_qualified_class_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // This is a native method, which enforces the argument count.
    let val = args[0];
    match val {
        Value::Null => return Ok("null".into()),
        Value::Undefined => return Ok("void".into()),
        _ => {}
    }
    let obj = val.coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class,
        None => match obj.instance_of() {
            Some(cls) => cls,
            None => return Ok(Value::Null),
        },
    };

    Ok(class
        .inner_class_definition()
        .read()
        .name()
        .to_qualified_name(activation.context.gc_context)
        .into())
}

/// Implements `flash.utils.getQualifiedSuperclassName`
pub fn get_qualified_superclass_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let obj = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let class = match obj.as_class_object() {
        Some(class) => class,
        None => match obj.instance_of() {
            Some(cls) => cls,
            None => return Ok(Value::Null),
        },
    };

    if let Some(super_class) = class.superclass_object() {
        Ok(super_class
            .inner_class_definition()
            .read()
            .name()
            .to_qualified_name(activation.context.gc_context)
            .into())
    } else {
        Ok(Value::Null)
    }
}

/// Implements native method `flash.utils.getDefinitionByName`
pub fn get_definition_by_name<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let appdomain = activation
        .caller_domain()
        .expect("Missing caller domain in getDefinitionByName");
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;
    appdomain.get_defined_value_handling_vector(activation, name)
}

// Implements `flash.utils.describeType`
pub fn describe_type<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = args[0].coerce_to_object(activation)?;
    let class_obj = value.as_class_object().or_else(|| value.instance_of());
    let Some(class_obj) = class_obj else {
        return Ok(activation
            .avm2()
            .classes()
            .xml
            .construct(activation, &[])?
            .into());
    };
    let mut xml_string = String::new();

    let is_static = value.as_class_object().is_some();

    let class = class_obj.inner_class_definition();
    let class = class.read();

    let qualified_name = class
        .name()
        .to_qualified_name(activation.context.gc_context);

    // If we're describing a Class object, then the "superclass" the the Class class
    let superclass = if is_static {
        Some(activation.avm2().classes().class)
    } else {
        class_obj.superclass_object()
    };

    let base_attr = if let Some(superclass) = superclass {
        format!(
            " base=\"{}\"",
            superclass
                .inner_class_definition()
                .read()
                .name()
                .to_qualified_name(activation.context.gc_context)
        )
    } else {
        String::new()
    };

    let is_dynamic = is_static || !class.is_sealed();
    let is_final = is_static || class.is_final();

    write!(xml_string, "<type name=\"{qualified_name}\"{base_attr} isDynamic=\"{is_dynamic}\" isFinal=\"{is_final}\" isStatic=\"{is_static}\">").unwrap();
    xml_string += &describe_internal_body(activation, class_obj, is_static)?;
    xml_string += "</type>";

    let xml_avm_string = AvmString::new_utf8(activation.context.gc_context, xml_string);

    Ok(activation
        .avm2()
        .classes()
        .xml
        .construct(activation, &[xml_avm_string.into()])?
        .into())
}

bitflags::bitflags! {
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
    class_obj: ClassObject<'gc>,
    is_static: bool,
) -> Result<String, Error<'gc>> {
    // This should be a const, but we can't use BitOr for bitflags in a const.
    let flash10_flags = DescribeTypeFlags::INCLUDE_BASES
        | DescribeTypeFlags::INCLUDE_INTERFACES
        | DescribeTypeFlags::INCLUDE_VARIABLES
        | DescribeTypeFlags::INCLUDE_ACCESSORS
        | DescribeTypeFlags::INCLUDE_METHODS
        | DescribeTypeFlags::INCLUDE_METADATA
        | DescribeTypeFlags::INCLUDE_CONSTRUCTOR
        | DescribeTypeFlags::INCLUDE_TRAITS
        | DescribeTypeFlags::HIDE_NSURI_METHODS
        | DescribeTypeFlags::HIDE_OBJECT;

    // FIXME - take this in from `avmplus.describeTypeJSON`
    let flags = flash10_flags;
    let mut xml_string = String::new();

    let class = class_obj.inner_class_definition();
    let class = class.read();

    let qualified_name = class
        .name()
        .to_qualified_name(activation.context.gc_context);

    // If we're describing a Class object, then the "superclass" the the Class class
    let superclass = if is_static {
        Some(activation.avm2().classes().class)
    } else {
        class_obj.superclass_object()
    };

    let mut current_super_obj = superclass;
    while let Some(super_obj) = current_super_obj {
        let super_name = super_obj
            .inner_class_definition()
            .read()
            .name()
            .to_qualified_name(activation.context.gc_context);
        write!(xml_string, "<extendsClass type=\"{super_name}\"/>").unwrap();
        current_super_obj = super_obj.superclass_object();
    }

    // When we're describing a Class object, we use the class vtable (which hides instance properties)
    let vtable = if is_static {
        class_obj.class_vtable()
    } else {
        class_obj.instance_vtable()
    };

    let super_vtable = if is_static {
        class_obj.instance_of().map(|c| c.instance_vtable())
    } else {
        class_obj.superclass_object().map(|c| c.instance_vtable())
    };

    if !is_static {
        for interface in class_obj.interfaces() {
            let interface_name = interface
                .read()
                .name()
                .to_qualified_name(activation.context.gc_context);
            write!(
                xml_string,
                "<implementsInterface type=\"{interface_name}\"/>"
            )
            .unwrap();
        }
    }

    // Implement the weird 'HIDE_NSURI_METHODS' behavior from avmplus:
    // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/TypeDescriber.cpp#L237
    let mut skip_ns = Vec::new();
    if let Some(super_vtable) = super_vtable {
        for (_, ns, prop) in super_vtable.resolved_traits().iter() {
            if !ns.as_uri().is_empty() {
                if let Property::Method { disp_id } = prop {
                    let method = super_vtable
                        .get_full_method(*disp_id)
                        .unwrap_or_else(|| panic!("Missing method for id {disp_id:?}"));
                    let is_playerglobals = method
                        .class
                        .class_scope()
                        .domain()
                        .is_playerglobals_domain(activation);

                    if !skip_ns.contains(&(ns, is_playerglobals)) {
                        skip_ns.push((ns, is_playerglobals));
                    }
                }
            }
        }
    }

    let class_is_playerglobals = class_obj
        .class_scope()
        .domain()
        .is_playerglobals_domain(activation);

    // FIXME - avmplus iterates over their own hashtable, so the order in the final XML
    // is different
    for (prop_name, ns, prop) in vtable.resolved_traits().iter() {
        if !ns.is_public_ignoring_ns() {
            continue;
        }

        // Hack around our lack of namespace versioning.
        // This is hack to work around the fact that we don't have namespace versioning
        // Once we do, methods from playerglobals should end up distinct public and AS3
        // namespaces, due to the special `kApiVersion_VM_ALLVERSIONS` used:
        // https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/AbcParser.cpp#L1497
        //
        // The main way this is
        // observable is by having a class like this:
        //
        // ``
        // class SubClass extends SuperClass {
        //   AS3 function subclassMethod {}
        // }
        // class SuperClass {}
        // ```
        //
        // Here, `subclassMethod` will not get hidden - even though `Object`
        // has AS3 methods, they are in the playerglobal AS3 namespace
        // (with version kApiVersion_VM_ALLVERSIONS), which is distinct
        // from the AS3 namespace used by SubClass. However, if we have any
        // user-defined classes in the inheritance chain, then the namespace
        // *should* match (if the swf version numbers match).
        //
        // For now, we approximate this by checking if the declaring class
        // and our starting class are both in the playerglobals domain
        // or both not in the playerglobals domain. If not, then we ignore
        // `skip_ns`, since we should really have two different namespaces here.
        if flags.contains(DescribeTypeFlags::HIDE_NSURI_METHODS)
            && skip_ns.contains(&(ns, class_is_playerglobals))
        {
            continue;
        }

        let uri = if ns.as_uri().is_empty() {
            String::new()
        } else {
            format!("uri=\"{}\"", ns.as_uri())
        };

        match prop {
            Property::ConstSlot { slot_id } | Property::Slot { slot_id } => {
                let prop_class_name = vtable
                    .slot_class_name(*slot_id, activation.context.gc_context)?
                    .to_qualified_name_or_star(activation.context.gc_context);

                let elem_name = match prop {
                    Property::ConstSlot { .. } => "constant",
                    Property::Slot { .. } => "variable",
                    _ => unreachable!(),
                };

                let trait_metadata = vtable.get_metadata_for_slot(slot_id);

                write!(
                    xml_string,
                    "<{elem_name} name=\"{prop_name}\" type=\"{prop_class_name}\">"
                )
                .unwrap();
                if let Some(metadata) = trait_metadata {
                    write_metadata(&mut xml_string, &metadata);
                }
                write!(xml_string, "</{elem_name}>").unwrap();
            }
            Property::Method { disp_id } => {
                let method = vtable
                    .get_full_method(*disp_id)
                    .unwrap_or_else(|| panic!("Missing method for id {disp_id:?}"));
                let return_type_name = method
                    .method
                    .return_type()
                    .to_qualified_name_or_star(activation.context.gc_context);
                let declared_by = method.class;

                if flags.contains(DescribeTypeFlags::HIDE_OBJECT)
                    && declared_by == activation.avm2().classes().object
                {
                    continue;
                }

                let declared_by_name = declared_by
                    .inner_class_definition()
                    .read()
                    .name()
                    .to_qualified_name(activation.context.gc_context);

                let trait_metadata = vtable.get_metadata_for_disp(disp_id);

                write!(xml_string, "<method name=\"{prop_name}\" declaredBy=\"{declared_by_name}\" returnType=\"{return_type_name}\" {uri}>").unwrap();
                write_params(&mut xml_string, &method.method, activation);
                if let Some(metadata) = trait_metadata {
                    write_metadata(&mut xml_string, &metadata);
                }
                xml_string += "</method>";
            }
            Property::Virtual { get, set } => {
                let access = match (get, set) {
                    (Some(_), Some(_)) => "readwrite",
                    (Some(_), None) => "readonly",
                    (None, Some(_)) => "writeonly",
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
                    (
                        setter.method.signature()[0].param_type_name.clone(),
                        setter.class,
                    )
                } else {
                    unreachable!();
                };

                let uri = if ns.as_uri().is_empty() {
                    String::new()
                } else {
                    format!("uri=\"{}\"", ns.as_uri())
                };

                let accessor_type =
                    method_type.to_qualified_name_or_star(activation.context.gc_context);
                let declared_by = defining_class
                    .inner_class_definition()
                    .read()
                    .name()
                    .to_qualified_name(activation.context.gc_context);

                write!(xml_string, "<accessor name=\"{prop_name}\" access=\"{access}\" type=\"{accessor_type}\" declaredBy=\"{declared_by}\" {uri}>").unwrap();

                if let Some(get_disp_id) = get {
                    if let Some(metadata) = vtable.get_metadata_for_disp(get_disp_id) {
                        write_metadata(&mut xml_string, &metadata);
                    }
                }

                if let Some(set_disp_id) = set {
                    if let Some(metadata) = vtable.get_metadata_for_disp(set_disp_id) {
                        write_metadata(&mut xml_string, &metadata);
                    }
                }

                write!(xml_string, "</accessor>").unwrap();
            }
        }
    }

    let constructor = class_obj.constructor();
    // Flash only shows a <constructor> element if it has at least one parameter
    if !is_static && !constructor.signature().is_empty() {
        xml_string += "<constructor>";
        write_params(&mut xml_string, &constructor, activation);
        xml_string += "</constructor>";
    }

    // If we're describing a Class object, add a <factory> element describing the instance.
    if is_static {
        write!(xml_string, "<factory type=\"{qualified_name}\">").unwrap();
        xml_string += &describe_internal_body(activation, class_obj, false)?;
        xml_string += "</factory>";
    }
    Ok(xml_string)
}

fn write_params<'gc>(
    xml_string: &mut String,
    method: &Method<'gc>,
    activation: &mut Activation<'_, 'gc>,
) {
    for (i, param) in method.signature().iter().enumerate() {
        let index = i + 1;
        let param_type_name = param
            .param_type_name
            .to_qualified_name_or_star(activation.context.gc_context);
        let optional = param.default_value.is_some();
        write!(
            xml_string,
            "<parameter index=\"{index}\" type=\"{param_type_name}\" optional=\"{optional}\"/>"
        )
        .unwrap();
    }
}

fn write_metadata(xml_string: &mut String, trait_metadata: &[Metadata<'_>]) {
    for single_trait in trait_metadata.iter() {
        write!(xml_string, "{}", single_trait.as_xml_string()).unwrap();
    }
}
