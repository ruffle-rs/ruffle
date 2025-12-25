use crate::avm2::activation::Activation;
use crate::avm2::call_stack::CallStack;
use crate::avm2::class::Class;
use crate::avm2::function::display_function;
use crate::avm2::method::Method;
use crate::avm2::multiname::Multiname;
use crate::avm2::object::{ClassObject, ErrorObject};
use crate::avm2::value::Value;
use crate::string::{AvmString, WString};

use quick_xml::errors::{Error as XmlError, SyntaxError as XmlSyntaxError};
use quick_xml::events::attributes::AttrError as XmlAttrError;
use ruffle_macros::istr;
use std::fmt::{Debug, Display};
use std::mem::size_of;

/// An error generated while handling AVM2 logic
#[repr(transparent)]
pub struct Error<'gc>(Box<ErrorData<'gc>>);

enum ErrorData<'gc> {
    /// A thrown error. This can be produced by an explicit 'throw' opcode.
    /// This can be caught by any catch blocks created by ActionScript code.
    AvmValue(Value<'gc>, CallStack<'gc>),

    /// A thrown `ErrorObject`. This can be produced by an explicit 'throw'
    /// opcode, or by a native implementation that throws an exception. This can
    /// be caught by any catch blocks created by ActionScript code. This is
    /// mostly equivalent to the `AvmValue` enum variant; the only difference is
    /// that the call stack used for `AvmError`s is the call stack of the
    /// `ErrorObject`, not a separately stored call stack.
    AvmError(ErrorObject<'gc>),

    /// An internal VM error. This cannot be caught by ActionScript code -
    /// it will either be logged by Ruffle, or cause the player to
    /// stop executing.
    RustError(Box<dyn std::error::Error>),
}

impl<'gc> Error<'gc> {
    fn new(data: ErrorData<'gc>) -> Self {
        Error(Box::new(data))
    }

    pub fn from_value(activation: &mut Activation<'_, 'gc>, error: Value<'gc>) -> Self {
        // If we're passed an `ErrorObject`, create an `ErrorData::AvmError`.
        // Otherwise, create an `ErrorData::AvmValue`.

        if let Some(error) = error.as_object().and_then(|o| o.as_error_object()) {
            Self::from_error_object(error)
        } else {
            let call_stack = activation.avm2().capture_call_stack();
            Error::new(ErrorData::AvmValue(error, call_stack))
        }
    }

    pub fn from_error_object(error: ErrorObject<'gc>) -> Self {
        Error::new(ErrorData::AvmError(error))
    }

    pub fn rust_error(error: Box<dyn std::error::Error>) -> Self {
        Error::new(ErrorData::RustError(error))
    }

    pub fn as_avm_error(&self) -> Option<Value<'gc>> {
        match &*self.0 {
            ErrorData::AvmValue(value, _) => Some(*value),
            ErrorData::AvmError(error) => Some((*error).into()),
            ErrorData::RustError(_) => None,
        }
    }
}

impl Debug for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (error, call_stack) = match &*self.0 {
            ErrorData::AvmValue(value, call_stack) => (*value, call_stack),
            ErrorData::AvmError(error) => ((*error).into(), error.call_stack()),
            ErrorData::RustError(error) => return write!(f, "RustError({error:?})"),
        };

        let mut output = WString::new();

        // If we have an `ErrorObject`, we can properly display it.
        // If not, just use the `Debug` impl for `Value`.
        // TODO: This should just call the `toString` method on the error value.
        if let Some(error) = error.as_object().and_then(|obj| obj.as_error_object()) {
            output.push_str(&error.display());
        } else {
            output.push_utf8(&format!("{:?}", error));
        }

        // Also write the call stack that was included with the error
        call_stack.display(&mut output);

        write!(f, "{}", output)
    }
}

// This type is used very frequently, so make sure it doesn't unexpectedly grow.
const _: () = assert!(size_of::<Result<Value<'_>, Error<'_>>>() <= 16);

macro_rules! make_error {
    ($expression:expr) => {
        Error::from_error_object($expression)
    };
}

#[inline(never)]
#[cold]
pub fn make_null_or_undefined_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
    name: Option<&Multiname<'gc>>,
) -> Error<'gc> {
    if matches!(value, Value::Undefined) {
        make_error_1010(activation, name)
    } else {
        let mut msg = "Error #1009: Cannot access a property or method of a null object reference."
            .to_string();
        if let Some(name) = name {
            msg.push_str(&format!(
                " (accessing field: {})",
                name.to_qualified_name(activation.gc())
            ));
        }
        make_error!(type_error(activation, &msg, 1009))
    }
}

pub enum ReferenceErrorCode {
    AssignToMethod = 1037,
    InvalidWrite = 1056,
    InvalidRead = 1069,
    WriteToReadOnly = 1074,
    ReadFromWriteOnly = 1077,
    InvalidNsRead = 1081,
    InvalidDelete = 1120,
}

#[inline(never)]
#[cold]
pub fn make_reference_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    code: ReferenceErrorCode,
    multiname: &Multiname<'gc>,
    object_class: Class<'gc>,
) -> Error<'gc> {
    let qualified_name = multiname.as_uri(activation.strings());
    let class_name = object_class
        .name()
        .to_qualified_name_err_message(activation.gc());

    let msg = match code {
        ReferenceErrorCode::AssignToMethod => format!(
            "Error #1037: Cannot assign to a method {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::InvalidWrite => format!(
            "Error #1056: Cannot create property {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::InvalidRead => format!(
            "Error #1069: Property {qualified_name} not found on {class_name} and there is no default value.",
        ),
        ReferenceErrorCode::WriteToReadOnly => format!(
            "Error #1074: Illegal write to read-only property {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::ReadFromWriteOnly => format!(
            "Error #1077: Illegal read of write-only property {qualified_name} on {class_name}.",
        ),
        ReferenceErrorCode::InvalidNsRead => format!(
            "Error #1081: Property {qualified_name} not found on {class_name} and there is no default value.",
        ),
        ReferenceErrorCode::InvalidDelete => format!(
            "Error #1120: Cannot delete property {qualified_name} on {class_name}.",
        ),
    };

    let class = activation.avm2().classes().referenceerror;
    make_error!(error_constructor(activation, class, &msg, code as u32))
}

#[inline(never)]
#[cold]
pub fn make_xml_error<'gc>(activation: &mut Activation<'_, 'gc>, err: XmlError) -> Error<'gc> {
    make_error!(match err {
        XmlError::InvalidAttr(XmlAttrError::Duplicated(_, _)) => type_error(
            activation,
            "Error #1104: Attribute was already specified for element.",
            1104,
        ),

        XmlError::Syntax(syntax_error) => match syntax_error {
            XmlSyntaxError::UnclosedCData => type_error(
                activation,
                "Error #1091: XML parser failure: Unterminated CDATA section.",
                1091,
            ),
            XmlSyntaxError::UnclosedDoctype => type_error(
                activation,
                "Error #1093: XML parser failure: Unterminated DOCTYPE declaration.",
                1093,
            ),
            XmlSyntaxError::UnclosedComment => type_error(
                activation,
                "Error #1094: XML parser failure: Unterminated comment.",
                1094,
            ),
            XmlSyntaxError::UnclosedPIOrXmlDecl => type_error(
                activation,
                "Error #1097: XML parser failure: Unterminated processing instruction.",
                1097,
            ),
            _ => type_error(
                activation,
                "Error #1090: XML parser failure: element is malformed.",
                1090,
            ),
        },
        _ => type_error(
            activation,
            "Error #1090: XML parser failure: element is malformed.",
            1090,
        ),
    })
}

#[inline(never)]
#[cold]
pub fn make_unknown_ns_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    ns: &[u8],
    local_name: AvmString<'gc>,
) -> Error<'gc> {
    make_error!(if ns.is_empty() {
        type_error(
            activation,
            &format!("Error #1084: Element or attribute (\":{local_name}\") does not match QName production: QName::=(NCName':')?NCName."),
            1084,
        )
    } else {
        // Note: Flash also uses this error message for attributes.
        type_error(
            activation,
            &format!(
                "Error #1083: The prefix \"{}\" for element \"{local_name}\" is not bound.",
                String::from_utf8_lossy(ns),
            ),
            1083,
        )
    })
}

#[inline(never)]
#[cold]
pub fn make_error_1002<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(range_error(
        activation,
        "Error #1002: Number.toPrecision has a range of 1 to 21. Number.toFixed and Number.toExponential have a range of 0 to 20. Specified value is not within expected range.",
        1002,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1003<'gc>(activation: &mut Activation<'_, 'gc>, radix: i32) -> Error<'gc> {
    make_error!(range_error(
        activation,
        &format!("Error #1003: The radix argument must be between 2 and 36; got {radix}."),
        1003,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1005<'gc>(activation: &mut Activation<'_, 'gc>, length: f64) -> Error<'gc> {
    make_error!(range_error(
        activation,
        &format!("Error #1005: Array index is not a positive integer ({length})."),
        1005,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1006<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1006: value is not a function.",
        1006
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1007<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1007: Instantiation attempted on a non-constructor.",
        1007,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1010<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: Option<&Multiname<'gc>>,
) -> Error<'gc> {
    let mut msg = "Error #1010: A term is undefined and has no properties.".to_string();
    if let Some(name) = name {
        msg.push_str(&format!(
            " (accessing field: {})",
            name.to_qualified_name(activation.gc())
        ));
    }
    make_error!(type_error(activation, &msg, 1010))
}

pub enum Error1014Type {
    ReferenceError,
    VerifyError,
}

#[inline(never)]
#[cold]
pub fn make_error_1011<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1011: Method contained illegal opcode.",
        1011,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1013<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1013: Cannot call OP_findproperty when scopeDepth is 0.",
        1013,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1014<'gc>(
    activation: &mut Activation<'_, 'gc>,
    kind: Error1014Type,
    class_name: AvmString<'gc>,
) -> Error<'gc> {
    let message = &format!("Error #1014: Class {class_name} could not be found.");
    make_error!(match kind {
        Error1014Type::ReferenceError => reference_error(activation, message, 1014),
        Error1014Type::VerifyError => verify_error(activation, message, 1014),
    })
}

#[inline(never)]
#[cold]
pub fn make_error_1016<'gc>(activation: &mut Activation<'_, 'gc>, class: Class<'gc>) -> Error<'gc> {
    let class_name = class.name().to_qualified_name_err_message(activation.gc());
    make_error!(type_error(
        activation,
        &format!("Error #1016: Descendants operator (..) not supported on type {class_name}",),
        1016,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1017<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1017: Scope stack overflow occurred.",
        1017,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1018<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1018: Scope stack underflow occurred.",
        1018,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1019<'gc>(
    activation: &mut Activation<'_, 'gc>,
    index: Option<usize>,
) -> Error<'gc> {
    let message = if let Some(index) = index {
        format!("Error #1019: Getscopeobject {index} is out of bounds.")
    } else {
        "Error #1019: Getscopeobject  is out of bounds.".to_string()
    };

    make_error!(verify_error(activation, &message, 1019))
}

#[inline(never)]
#[cold]
pub fn make_error_1020<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1020: Code cannot fall off the end of a method.",
        1020,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1021<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1021: At least one branch target was not on a valid instruction in the method.",
        1021,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1023<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1023: Stack overflow occurred.",
        1023,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1024<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1024: Stack underflow occurred.",
        1024,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1025<'gc>(activation: &mut Activation<'_, 'gc>, index: u32) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        &format!("Error #1025: An invalid register {index} was accessed."),
        1025,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1026<'gc>(
    activation: &mut Activation<'_, 'gc>,
    slot_id: u32,
    slot_count: Option<usize>,
    class: Option<Class<'gc>>,
) -> Error<'gc> {
    let message = if let (Some(slot_count), Some(class)) = (slot_count, class) {
        let class_name = class.name().to_qualified_name_err_message(activation.gc());

        format!("Error #1026: Slot {slot_id} exceeds slotCount={slot_count} of {class_name}.")
    } else {
        format!("Error #1026: Slot {slot_id} exceeds slotCount.")
    };
    make_error!(verify_error(activation, &message, 1026))
}

#[inline(never)]
#[cold]
pub fn make_error_1027<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1027: Method_info exceeds method_count.",
        1027,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1030<'gc>(
    activation: &mut Activation<'_, 'gc>,
    first_len: usize,
    second_len: usize,
) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        &format!(
            "Error #1030: Stack depth is unbalanced. {} != {}.",
            first_len, second_len,
        ),
        1030,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1031<'gc>(
    activation: &mut Activation<'_, 'gc>,
    first_len: usize,
    second_len: usize,
) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        &format!(
            "Error #1031: Scope depth is unbalanced. {} != {}.",
            first_len, second_len,
        ),
        1031,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1032<'gc>(activation: &mut Activation<'_, 'gc>, index: u32) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        &format!("Error #1032: Cpool index {index} is out of range."),
        1032,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1033<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1033: Cpool entry is wrong type.",
        1033
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1034<'gc>(
    activation: &mut Activation<'_, 'gc>,
    value: Value<'gc>,
    target_class: Class<'gc>,
) -> Error<'gc> {
    let debug_str = match value.as_debug_string(activation) {
        Ok(string) => string,
        Err(err) => return err,
    };

    let class_name = target_class
        .name()
        .to_qualified_name_err_message(activation.gc());

    make_error!(type_error(
        activation,
        &format!("Error #1034: Type Coercion failed: cannot convert {debug_str} to {class_name}."),
        1034,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1035<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1035: Illegal super expression found in method.",
        1035,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1040<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1040: The right-hand side of instanceof must be a class or function.",
        1040,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1041<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1041: The right-hand side of operator must be a class.",
        1041,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1043<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1043: Invalid code_length=0.",
        1043
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1047<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1047: No entry point was found.",
        1047
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1050<'gc>(activation: &mut Activation<'_, 'gc>, value: Value<'gc>) -> Error<'gc> {
    let class_name = value.instance_of_class_name(activation);

    make_error!(type_error(
        activation,
        &format!("Error #1050: Cannot convert {class_name} to primitive."),
        1050,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1051<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1051: Illegal early binding access.",
        1051,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1052<'gc>(activation: &mut Activation<'_, 'gc>, func_name: &str) -> Error<'gc> {
    make_error!(uri_error(
        activation,
        &format!("Error #1052: Invalid URI passed to {func_name} function."),
        1052,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1053<'gc>(
    activation: &mut Activation<'_, 'gc>,
    trait_name: AvmString<'gc>,
    class_name: AvmString<'gc>,
) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        &format!("Error #1053: Illegal override of {trait_name} in {class_name}."),
        1053,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1054<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1054: Illegal range or target offsets in exception handler.",
        1054,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1058<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "#1058: Illegal operand type.",
        1058
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1059<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1059: ClassInfo is referenced before definition.",
        1059,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1063<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
    passed_arg_count: usize,
) -> Error<'gc> {
    let expected_num_params = method
        .signature()
        .iter()
        .filter(|param| param.default_value.is_none())
        .count();

    let mut function_name = WString::new();

    display_function(&mut function_name, method);

    make_error!(argument_error(
        activation,
        &format!(
            "Error #1063: Argument count mismatch on {function_name}. Expected {expected_num_params}, got {passed_arg_count}.",
        ),
        1063,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1064<'gc>(
    activation: &mut Activation<'_, 'gc>,
    method: Method<'gc>,
) -> Error<'gc> {
    let mut function_name = WString::new();

    display_function(&mut function_name, method);

    make_error!(type_error(
        activation,
        &format!("Error #1064: Cannot call method {function_name} as constructor.",),
        1064,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1065<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: &Multiname<'gc>,
) -> Error<'gc> {
    // FIXME: in FP, sometimes this uses the full qualified name, rather than
    // just the local name
    let local_name = name.local_name().unwrap_or(istr!("*"));

    make_error!(reference_error(
        activation,
        &format!("Error #1065: Variable {local_name} is not defined."),
        1065,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1066<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(eval_error(
        activation,
        "Error #1066: The form function('function body') is not supported.",
        1066,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1068<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1068: Scope values cannot be reconciled.",
        1068,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1070<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: Class<'gc>,
    multiname: &Multiname<'gc>,
) -> Error<'gc> {
    let class_name = class.name().to_qualified_name_err_message(activation.gc());
    let multiname_name = multiname.as_uri(activation.strings());

    make_error!(reference_error(
        activation,
        &format!("Error #1070: Method {multiname_name} not found on {class_name}",),
        1070,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1072<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1072: Disp_id 0 is illegal.",
        1072
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1075<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1075: Math is not a function.",
        1075,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1076<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1076: Math is not a constructor.",
        1076,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1078<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1078: Illegal opcode/multiname combination.",
        1078,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1080<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1080: Illegal value for namespace.",
        1080,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1085<'gc>(activation: &mut Activation<'_, 'gc>, tag: &str) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!("Error #1085: The element type \"{tag}\" must be terminated by the matching end-tag \"</{tag}>\"."),
        1085,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1086<'gc>(activation: &mut Activation<'_, 'gc>, method_name: &str) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!("Error #1086: The {method_name} method only works on lists containing one item."),
        1086,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1087<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1087: Assignment to indexed XML is not allowed.",
        1087,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1088<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1088: The markup in the document following the root element must be well-formed.",
        1088,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1089<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1089: Assignment to lists with more than one item is not supported.",
        1089,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1098<'gc>(
    activation: &mut Activation<'_, 'gc>,
    prefix: AvmString<'gc>,
) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!("Error #1098: Illegal prefix {prefix} for no namespace."),
        1098,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1100<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1100: Cannot supply flags when constructing one RegExp from another.",
        1100,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1103<'gc>(activation: &mut Activation<'_, 'gc>, class: Class<'gc>) -> Error<'gc> {
    let class_name = class.name().to_qualified_name(activation.gc());

    make_error!(verify_error(
        activation,
        &format!(
            "Error #1103: Class {} cannot extend final base class.",
            class_name
        ),
        1103,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1107<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1107: The ABC data is corrupt, attempt to read out of bounds.",
        1107,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1108<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1108: The OP_newclass opcode was used with the incorrect base class.",
        1108,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1110<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: Class<'gc>,
    super_class: Class<'gc>,
) -> Error<'gc> {
    let mc = activation.gc();

    let class_name = class.name().to_qualified_name(mc);
    let super_class_name = super_class.name().to_qualified_name_err_message(mc);

    make_error!(verify_error(
        activation,
        &format!(
            "Error #1110: Class {} cannot extend {}.",
            class_name, super_class_name
        ),
        1110,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1111<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: Class<'gc>,
    interface: Class<'gc>,
) -> Error<'gc> {
    let mc = activation.gc();

    let class_name = class.name().to_qualified_name(mc);
    let interface_name = interface.name().to_qualified_name_err_message(mc);

    make_error!(verify_error(
        activation,
        &format!(
            "Error #1111: {} cannot implement {}.",
            class_name, interface_name
        ),
        1111,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1112<'gc>(activation: &mut Activation<'_, 'gc>, arg_count: usize) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!(
            "Error #1112: Argument count mismatch on class coercion.  Expected 1, got {}.",
            arg_count
        ),
        1112,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1113<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1113: OP_newactivation used in method without NEED_ACTIVATION flag.",
        1113,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1115<'gc>(activation: &mut Activation<'_, 'gc>, name: &str) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!("Error #1115: {name} is not a constructor."),
        1115,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1116<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1116: second argument to Function.prototype.apply must be an array.",
        1116,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1117<'gc>(
    activation: &mut Activation<'_, 'gc>,
    name: AvmString<'gc>,
) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!("Error #1117: Invalid XML name: {}.", name.as_wstr()),
        1117,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1118<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1118: Illegal cyclical loop between nodes.",
        1118,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1119<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1119: Delete operator is not supported with operand of type XMLList.",
        1119,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1123<'gc>(activation: &mut Activation<'_, 'gc>, class: Class<'gc>) -> Error<'gc> {
    let class_name = class.name().to_qualified_name_err_message(activation.gc());

    make_error!(type_error(
        activation,
        &format!("Error #1123: Filter operator not supported on type {class_name}."),
        1123,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1124<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(verify_error(
        activation,
        "Error #1124: OP_hasnext2 requires object and index to be distinct registers.",
        1124,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1125<'gc>(
    activation: &mut Activation<'_, 'gc>,
    index: f64,
    range: usize,
) -> Error<'gc> {
    make_error!(range_error(
        activation,
        &format!("Error #1125: The index {index} is out of range {range}."),
        1125,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1126<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(range_error(
        activation,
        "Error #1126: Cannot change the length of a fixed Vector.",
        1126,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1127<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1127: Type application attempted on a non-parameterized type.",
        1127,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1128<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: Class<'gc>,
    param_count: usize,
) -> Error<'gc> {
    let class_name = class.name().to_qualified_name_err_message(activation.gc());

    make_error!(type_error(
        activation,
        &format!(
            "Error #1128: Incorrect number of type parameters for {}. Expected 1, got {}.",
            class_name, param_count
        ),
        1128,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1129<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1129: Cyclic structure cannot be converted to JSON string.",
        1129,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1131<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        "Error #1131: Replacer argument to JSON stringifier must be an array or a two parameter function.",
        1131,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1132<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(syntax_error(
        activation,
        "Error #1132: Invalid JSON parse input.",
        1132
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1504<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(activation, "Error #1504: End of file.", 1504))
}

#[inline(never)]
#[cold]
pub fn make_error_1506<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(range_error(
        activation,
        "Error #1506: The range specified is invalid.",
        1506,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1507<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #1507: Argument {param_name} cannot be null."),
        1507,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_1508<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #1508: The value specified for argument {param_name} is invalid."),
        1508,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2002<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(io_error(
        activation,
        "Error #2002: Operation attempted on invalid socket.",
        2002,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2003<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(security_error(
        activation,
        "Error #2003: Invalid socket port number specified.",
        2003,
    ))
}

pub enum Error2004Type {
    Error,
    ArgumentError,
    TypeError,
}

#[inline(never)]
#[cold]
pub fn make_error_2004<'gc>(
    activation: &mut Activation<'_, 'gc>,
    kind: Error2004Type,
) -> Error<'gc> {
    let message = "Error #2004: One of the parameters is invalid.";
    make_error!(match kind {
        Error2004Type::Error => error(activation, message, 2004),
        Error2004Type::ArgumentError => argument_error(activation, message, 2004),
        Error2004Type::TypeError => type_error(activation, message, 2004),
    })
}

#[inline(never)]
#[cold]
pub fn make_error_2005<'gc>(
    activation: &mut Activation<'_, 'gc>,
    param_index: u32,
    param_name: &str,
) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #2005: Parameter {param_index} is of the incorrect type. Should be type {param_name}."),
        2005,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2006<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(range_error(
        activation,
        "Error #2006: The supplied index is out of bounds.",
        2006,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2007<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!("Error #2007: Parameter {param_name} must be non-null."),
        2007,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2008<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #2008: Parameter {param_name} must be one of the accepted values."),
        2008,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2012<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class_name: impl Display,
) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #2012: {class_name} class cannot be instantiated."),
        2012,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2015<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2015: Invalid BitmapData.",
        2015
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2022<'gc>(activation: &mut Activation<'_, 'gc>, class: Class<'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!(
            "Error #2022: Class {}$ must inherit from DisplayObject to link to a symbol.",
            class.name().local_name()
        ),
        2022,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2023<'gc>(activation: &mut Activation<'_, 'gc>, class: Class<'gc>) -> Error<'gc> {
    make_error!(type_error(
        activation,
        &format!(
            "Error #2023: Class {}$ must inherit from Sprite to link to the root.",
            class.name().local_name()
        ),
        2023,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2024<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2024: An object cannot be added as a child of itself.",
        2024,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2025<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2025: The supplied DisplayObject must be a child of the caller.",
        2025,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2027<'gc>(
    activation: &mut Activation<'_, 'gc>,
    param_name: &str,
    value: i32,
) -> Error<'gc> {
    make_error!(range_error(
        activation,
        &format!("Error #2027: Parameter {param_name} must be a non-negative number; got {value}."),
        2027,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2030<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(eof_error(
        activation,
        "Error #2030: End of file was encountered.",
        2030,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2037<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2037: Functions called in incorrect sequence, or earlier call was unsuccessful.",
        2037,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2058<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(io_error(
        activation,
        "Error #2058: There was an error decompressing the data.",
        2058,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2067<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2067: The ExternalInterface is not available in this container. ExternalInterface requires Internet Explorer ActiveX, Firefox, Mozilla 1.7.5 and greater, or other browsers that support NPRuntime.",
        2067,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2078<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(illegal_operation_error(
        activation,
        "Error #2078: The name property of a Timeline-placed object cannot be modified.",
        2078,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2082<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2082: Connect failed because the object is already connected.",
        2082,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2083<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2083: Close failed because the object is not connected.",
        2083,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2084<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2084: The AMF encoding of the arguments cannot exceed 40K.",
        2084,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2085<'gc>(activation: &mut Activation<'_, 'gc>, param_name: &str) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #2085: Parameter {param_name} must be non-empty string."),
        2007,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2097<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2097: The FileFilter Array is not in the correct format.",
        2097,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2099<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2099: The loading object is not sufficiently loaded to provide this information.",
        2099,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2109<'gc>(
    activation: &mut Activation<'_, 'gc>,
    frame_label: AvmString<'gc>,
    scene: AvmString<'gc>,
) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #2109: Frame label {frame_label} not found in scene {scene}."),
        2109,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2126<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2126: NetConnection object must be connected.",
        2126,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2130<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2130: Unable to flush SharedObject.",
        2130,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2136<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2136: The SWF file contains invalid data.",
        2136,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2150<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2150: An object cannot be added as a child to one of it's children (or children's children, etc.).",
        2150,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2162<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2162: The Shader output type is not compatible for this operation.",
        2162,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2165<'gc>(activation: &mut Activation<'_, 'gc>, input_name: &str) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!(
            "Error #2165: The Shader input {} does not have enough data.",
            input_name
        ),
        2165,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2174<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2174: Only one download, upload, load or save operation can be active at a time on each FileReference.",
        2174,
    ))
}

// Currently we don't use this, see `globals::flash::system::system::set_clipboard`
#[allow(dead_code)]
#[inline(never)]
#[cold]
pub fn make_error_2176<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #2176: Certain actions, such as those that display a pop-up window, may only be invoked upon user interaction, for example by a mouse click or button press.",
        2176,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2180<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2180: It is illegal to move AVM1 content (AS1 or AS2) to a different part of the displayList when it has been loaded into AVM2 (AS3) content.",
        2180,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2182<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #2182: Invalid fieldOfView value.  The value must be greater than 0 and less than 180.",
        2182,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_2186<'gc>(activation: &mut Activation<'_, 'gc>, focal_length: f64) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        &format!("Error #2186: Invalid focalLength {focal_length}."),
        2186,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3669<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(activation, "Error #3669: Bad input size.", 3669))
}

#[inline(never)]
#[cold]
pub fn make_error_3670<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3670: Buffer too big.",
        3670,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3671<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3671: Buffer has zero size.",
        3671,
    ))
}

// This isn't used if the `jpegxr` feature is disabled, see
// `globals::flash::display3D::textures::atf_jpegxr`
#[allow(dead_code)]
#[inline(never)]
#[cold]
pub fn make_error_3675<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3675: Texture format mismatch.",
        3675,
    ))
}

// This isn't used if the `jpegxr` feature is disabled, see
// `globals::flash::display3D::textures::atf_jpegxr`
#[allow(dead_code)]
#[inline(never)]
#[cold]
pub fn make_error_3679<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3679: Texture size does not match.",
        3679,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3771<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3771: 2D textures need to have surfaceSelector = 0.",
        3771,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3772<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3772: Cube textures need to have surfaceSelector [0..5].",
        3772,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3773<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3773: Rectangle textures need to have surfaceSelector = 0.",
        3773,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3780<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #3780: Requested width of backbuffer is not in allowed range 32 to 16384.",
        3680,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3781<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(error(
        activation,
        "Error #3781: Requested height of backbuffer is not in allowed range 32 to 16384.",
        3681,
    ))
}

#[inline(never)]
#[cold]
pub fn make_error_3783<'gc>(activation: &mut Activation<'_, 'gc>) -> Error<'gc> {
    make_error!(argument_error(
        activation,
        "Error #3783: A Stage object cannot be added as the child of another object.",
        3783,
    ))
}

#[inline(never)]
#[cold]
fn range_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().rangeerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn eval_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().evalerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn argument_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().argumenterror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn security_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().securityerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn type_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().typeerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn reference_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().referenceerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn verify_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().verifyerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn illegal_operation_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().illegaloperationerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn io_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().ioerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn eof_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().eoferror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn uri_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().urierror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn syntax_error<'gc>(
    activation: &mut Activation<'_, 'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().syntaxerror;
    error_constructor(activation, class, message, code)
}

#[inline(never)]
#[cold]
fn error<'gc>(activation: &mut Activation<'_, 'gc>, message: &str, code: u32) -> ErrorObject<'gc> {
    let class = activation.avm2().classes().error;
    error_constructor(activation, class, message, code)
}

fn error_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class: ClassObject<'gc>,
    message: &str,
    code: u32,
) -> ErrorObject<'gc> {
    let message = AvmString::new_utf8(activation.gc(), message);

    ErrorObject::from_info(activation, class, message, code)
}

impl std::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// Ideally, all of these impls would be unified under a single
// `impl<E: std::error::Error> From<E> for Error<'gc>`
// However, this would conflict with the 'str' and 'String'
// impls, which are still widely used.

impl<'gc, 'a> From<&'a str> for Error<'gc> {
    fn from(val: &'a str) -> Error<'gc> {
        Error::rust_error(val.into())
    }
}

impl<'gc> From<String> for Error<'gc> {
    fn from(val: String) -> Error<'gc> {
        Error::rust_error(val.into())
    }
}

impl<'gc> From<ruffle_render::error::Error> for Error<'gc> {
    fn from(val: ruffle_render::error::Error) -> Error<'gc> {
        Error::rust_error(val.into())
    }
}
