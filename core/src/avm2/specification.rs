use crate::avm2::dynamic_map::DynamicKey;
use crate::avm2::function::BoundMethod;
use crate::avm2::method::{Method, ParamConfig};
use crate::avm2::object::TObject;
use crate::avm2::traits::{Trait, TraitKind};
use crate::avm2::{Activation, Avm2, ClassObject, QName, Value};
use crate::context::UpdateContext;
use crate::string::AvmString;
use crate::stub::Stub;
use fnv::{FnvHashMap, FnvHashSet};
use serde::Serialize;
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;
use std::process::exit;

fn is_false(b: &bool) -> bool {
    !(*b)
}

fn escape_string(string: &AvmString) -> String {
    let mut output = "".to_string();
    output.push('\"');

    for c in string.chars() {
        let c = c.unwrap_or(char::REPLACEMENT_CHARACTER);
        let escape = match u8::try_from(c as u32) {
            Ok(b'"') => "\\\"",
            Ok(b'\\') => "\\\\",
            Ok(b'\n') => "\\n",
            Ok(b'\r') => "\\r",
            Ok(b'\t') => "\\t",
            Ok(0x08) => "\\b",
            Ok(0x0C) => "\\f",
            _ => {
                output.push(c);
                continue;
            }
        };

        output.push_str(escape);
    }

    output.push('\"');
    output
}

fn format_value(value: &Value) -> Option<String> {
    match value {
        Value::Undefined => None,
        Value::Null => Some("null".to_string()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::Integer(value) => Some(value.to_string()),
        Value::String(value) => Some(escape_string(value)),
        Value::Object(_) => None,
    }
}

fn format_signature(params: &[ParamConfig], is_variadic: bool) -> Vec<ParamInfo> {
    let mut result = Vec::with_capacity(params.len());

    for param in params {
        result.push(ParamInfo {
            type_info: param
                .param_type_name
                .and_then(|m| m.local_name())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "*".to_string()),
            value: param.default_value.and_then(|v| format_value(&v)),
            variadic: false,
        });
    }

    if is_variadic {
        result.push(ParamInfo {
            type_info: "*".to_string(),
            value: None,
            variadic: true,
        })
    }

    result
}

#[derive(Serialize, Default)]
struct ClassInfo {
    #[serde(skip_serializing_if = "is_false")]
    dynamic: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    extends: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    implements: Option<String>,

    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "final")]
    is_final: bool,
}

#[derive(Serialize, Default)]
struct VariableInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    type_info: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,

    #[serde(skip_serializing_if = "is_false")]
    stubbed: bool,
}

impl VariableInfo {
    pub fn from_value<'gc>(value: Value<'gc>, activation: &mut Activation<'_, 'gc>) -> Self {
        Self {
            type_info: match value {
                Value::Bool(_) => Some("Boolean".to_string()),
                Value::Number(_) => Some("Number".to_string()),
                Value::Integer(_) => Some("int".to_string()),
                Value::String(_) => Some("String".to_string()),
                Value::Object(_) => Some("Object".to_string()),
                _ => Some("*".to_string()),
            },
            value: value
                .coerce_to_string(activation)
                .ok()
                .map(|v| v.to_string()),
            stubbed: false,
        }
    }
}

#[derive(Serialize, Default)]
struct ParamInfo {
    #[serde(rename = "type")]
    type_info: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "default")]
    value: Option<String>,

    #[serde(skip_serializing_if = "is_false")]
    variadic: bool,
}

#[derive(Serialize)]
struct FunctionInfo {
    args: Vec<ParamInfo>,
    returns: String,
    #[serde(skip_serializing_if = "is_false")]
    stubbed: bool,
}

impl FunctionInfo {
    pub fn from_method(method: &Method, stubbed: bool) -> Self {
        Self {
            returns: method
                .return_type()
                .and_then(|m| m.local_name())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "void".to_string()),
            args: format_signature(method.signature(), method.is_variadic()),
            stubbed,
        }
    }

    pub fn from_bound_method(executable: &BoundMethod, stubbed: bool) -> Self {
        Self {
            returns: executable
                .return_type()
                .and_then(|m| m.local_name())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "void".to_string()),
            args: format_signature(executable.signature(), executable.is_variadic()),
            stubbed,
        }
    }
}

#[derive(Serialize, Default)]
struct TraitList {
    #[serde(rename = "const")]
    #[serde(skip_serializing_if = "FnvHashMap::is_empty")]
    constants: FnvHashMap<String, VariableInfo>,

    #[serde(rename = "var")]
    #[serde(skip_serializing_if = "FnvHashMap::is_empty")]
    variables: FnvHashMap<String, VariableInfo>,

    #[serde(skip_serializing_if = "FnvHashMap::is_empty")]
    function: FnvHashMap<String, FunctionInfo>,

    #[serde(skip_serializing_if = "FnvHashMap::is_empty")]
    getter: FnvHashMap<String, VariableInfo>,

    #[serde(skip_serializing_if = "FnvHashMap::is_empty")]
    setter: FnvHashMap<String, VariableInfo>,
}

#[derive(Serialize, Default)]
struct Definition {
    #[serde(skip_serializing_if = "Option::is_none")]
    classinfo: Option<ClassInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "static")]
    static_traits: Option<TraitList>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "instance")]
    instance_traits: Option<TraitList>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "prototype")]
    prototype: Option<TraitList>,
}

#[derive(Default)]
struct ClassStubs {
    methods: FnvHashSet<Cow<'static, str>>,
    getters: FnvHashSet<Cow<'static, str>>,
    setters: FnvHashSet<Cow<'static, str>>,
}

impl ClassStubs {
    pub fn for_class(class_name: &str, stubs: &FnvHashSet<&Stub>) -> Self {
        let mut result = ClassStubs::default();

        for stub in stubs
            .iter()
            .filter(|s| s.avm2_class() == Some(Cow::Borrowed(class_name)))
        {
            match stub {
                Stub::Avm2Method { method, .. } => {
                    result.methods.insert(method.clone());
                }
                Stub::Avm2Getter { property, .. } => {
                    result.getters.insert(property.clone());
                }
                Stub::Avm2Setter { property, .. } => {
                    result.setters.insert(property.clone());
                }
                _ => {}
            }
        }

        result
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.methods.contains(name)
    }

    pub fn has_getter(&self, name: &str) -> bool {
        self.getters.contains(name)
    }

    pub fn has_setter(&self, name: &str) -> bool {
        self.setters.contains(name)
    }
}

impl Definition {
    fn from_class<'gc>(
        class_object: ClassObject<'gc>,
        activation: &mut Activation<'_, 'gc>,
        stubs: &ClassStubs,
    ) -> Self {
        let mut definition = Self::default();
        let i_class = class_object.inner_class_definition();
        let c_class = i_class
            .c_class()
            .expect("inner_class_definition should be an i_class");

        if i_class.is_final() {
            definition
                .classinfo
                .get_or_insert_with(Default::default)
                .is_final = true;
        }
        if !i_class.is_sealed() {
            definition
                .classinfo
                .get_or_insert_with(Default::default)
                .dynamic = true;
        }
        if let Some(super_name) = i_class
            .super_class_name()
            .as_ref()
            .and_then(|n| n.local_name())
        {
            if &super_name != b"Object" {
                definition
                    .classinfo
                    .get_or_insert_with(Default::default)
                    .extends = Some(super_name.to_string());
            }
        }

        let prototype = class_object.prototype();
        let prototype_base = prototype.base();
        let prototype_values = prototype_base.values();
        for (key, value) in prototype_values.as_hashmap().iter() {
            let name = match key {
                DynamicKey::String(name) => *name,
                DynamicKey::Uint(key) => {
                    AvmString::new_utf8(activation.context.gc_context, key.to_string())
                }
                DynamicKey::Object(object) => {
                    Value::Object(*object).coerce_to_string(activation).unwrap()
                }
            };
            if &name != b"constructor" {
                Self::add_prototype_value(
                    &name,
                    value.value,
                    &mut definition.prototype,
                    activation,
                );
            }
        }

        Self::fill_traits(
            activation.avm2(),
            &c_class.traits(),
            &mut definition.static_traits,
            stubs,
        );
        Self::fill_traits(
            activation.avm2(),
            &i_class.traits(),
            &mut definition.instance_traits,
            stubs,
        );

        definition
    }

    fn add_prototype_value<'gc>(
        name: &AvmString<'gc>,
        value: Value<'gc>,
        output: &mut Option<TraitList>,
        activation: &mut Activation<'_, 'gc>,
    ) {
        if let Some(object) = value.as_object() {
            if let Some(executable) = object.as_executable() {
                output.get_or_insert_with(Default::default).function.insert(
                    name.to_string(),
                    FunctionInfo::from_bound_method(&executable, false),
                );
            }
        } else {
            output
                .get_or_insert_with(Default::default)
                .variables
                .insert(
                    name.to_string(),
                    VariableInfo::from_value(value, activation),
                );
        }
    }

    fn fill_traits<'gc>(
        avm2: &Avm2<'gc>,
        traits: &[Trait<'gc>],
        output: &mut Option<TraitList>,
        stubs: &ClassStubs,
    ) {
        for class_trait in traits {
            if !class_trait.name().namespace().is_public()
                && !class_trait
                    .name()
                    .namespace()
                    .exact_version_match(avm2.namespaces.as3)
            {
                continue;
            }
            let trait_name = class_trait.name().local_name().to_string();
            match class_trait.kind() {
                TraitKind::Slot {
                    type_name,
                    default_value,
                    ..
                } => {
                    output
                        .get_or_insert_with(Default::default)
                        .variables
                        .insert(
                            trait_name,
                            VariableInfo {
                                type_info: type_name
                                    .and_then(|m| m.local_name())
                                    .map(|n| n.to_string()),
                                value: format_value(default_value),
                                stubbed: false,
                            },
                        );
                }
                TraitKind::Method { method, .. } => {
                    let stubbed = stubs.has_method(&trait_name);
                    output
                        .get_or_insert_with(Default::default)
                        .function
                        .insert(trait_name, FunctionInfo::from_method(method, stubbed));
                }
                TraitKind::Getter { method, .. } => {
                    let stubbed = stubs.has_getter(&trait_name);
                    output.get_or_insert_with(Default::default).getter.insert(
                        trait_name,
                        VariableInfo {
                            type_info: Some(
                                method
                                    .return_type()
                                    .and_then(|m| m.local_name())
                                    .map(|n| n.to_string())
                                    .unwrap_or_else(|| "*".to_string()),
                            ),
                            value: None,
                            stubbed,
                        },
                    );
                }
                TraitKind::Setter { method, .. } => {
                    let stubbed = stubs.has_setter(&trait_name);
                    output.get_or_insert_with(Default::default).setter.insert(
                        trait_name,
                        VariableInfo {
                            type_info: Some(
                                method
                                    .signature()
                                    .first()
                                    .and_then(|p| p.param_type_name)
                                    .and_then(|m| m.local_name())
                                    .map(|t| t.to_string())
                                    .unwrap_or_else(|| "*".to_string()),
                            ),
                            value: None,
                            stubbed,
                        },
                    );
                }
                TraitKind::Class { .. } => {}
                TraitKind::Function { .. } => {}
                TraitKind::Const {
                    type_name,
                    default_value,
                    ..
                } => {
                    output
                        .get_or_insert_with(Default::default)
                        .constants
                        .insert(
                            trait_name,
                            VariableInfo {
                                type_info: type_name
                                    .and_then(|m| m.local_name())
                                    .map(|n| n.to_string()),
                                value: format_value(default_value),
                                stubbed: false,
                            },
                        );
                }
            }
        }
    }
}

#[allow(unreachable_code, unused_variables, clippy::diverging_sub_expression)]
pub fn capture_specification(context: &mut UpdateContext, output: &Path) {
    let stubs = crate::stub::get_known_stubs();

    let mut definitions = FnvHashMap::<String, Definition>::default();

    let defs = context.avm2.playerglobals_domain.defs().clone();
    let mut activation = Activation::from_nothing(context);
    for (name, namespace, _) in defs.iter() {
        let value = activation
            .context
            .avm2
            .playerglobals_domain
            .get_defined_value(&mut activation, QName::new(namespace, name))
            .expect("Builtins shouldn't error");
        if let Some(object) = value.as_object() {
            if let Some(class) = object.as_class_object() {
                let class_name = class
                    .inner_class_definition()
                    .name()
                    .to_qualified_name_err_message(activation.context.gc_context)
                    .to_string();
                let class_stubs = ClassStubs::for_class(&class_name, &stubs);
                definitions.insert(
                    class_name,
                    Definition::from_class(class, &mut activation, &class_stubs),
                );
            } else if let Some(executable) = object.as_executable() {
                let namespace_stubs =
                    ClassStubs::for_class(&namespace.as_uri().to_string(), &stubs);
                let definition = definitions
                    .entry(namespace.as_uri().to_string())
                    .or_default();
                let instance_traits = definition
                    .instance_traits
                    .get_or_insert_with(Default::default);
                instance_traits.function.insert(
                    name.to_string(),
                    FunctionInfo::from_bound_method(
                        &executable,
                        namespace_stubs.has_method(&name.to_string()),
                    ),
                );
            }
        } else {
            let definition = definitions
                .entry(namespace.as_uri().to_string())
                .or_default();
            let instance_traits = definition
                .instance_traits
                .get_or_insert_with(Default::default);
            instance_traits.constants.insert(
                name.to_string(),
                VariableInfo::from_value(value, &mut activation),
            );
        }
    }
    serde_json::to_writer_pretty(&File::create(output).unwrap(), &definitions).unwrap();
    tracing::info!("Wrote stub report to {output:?}");
    exit(0);
}
