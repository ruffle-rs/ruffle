//! An internal Ruffle utility to build our playerglobal
//! `library.swf`

use convert_case::{Boundary, Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use regex::RegexBuilder;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use swf::avm2::read::Reader;
use swf::avm2::types::*;
use swf::avm2::write::Writer;
use swf::extensions::ReadSwfExt;
use swf::{DoAbc2, DoAbc2Flag, Header, Tag};
use walkdir::WalkDir;

// The metadata name - all metadata in our .as files
// should be of the form `[Ruffle(key1 = value1, key2 = value2)]`
const RUFFLE_METADATA_NAME: &str = "Ruffle";
// Indicates that we should generate a reference to an instance allocator
// method (used as a metadata key with `Ruffle` metadata)
const METADATA_INSTANCE_ALLOCATOR: &str = "InstanceAllocator";
/// Indicates that we should generate a reference to a class call handler
/// method (used as a metadata key with `Ruffle` metadata)
const METADATA_CALL_HANDLER: &str = "CallHandler";
/// Indicates that we should generate a class call handler that constructs the
/// class being called.
const METADATA_CONSTRUCT_ON_CALL: &str = "ConstructOnCall";
/// Indicates that we should generate a reference to a custom constructor
/// method (used as a metadata key with `Ruffle` metadata)
const METADATA_CUSTOM_CONSTRUCTOR: &str = "CustomConstructor";
/// Indicates that the class can't be directly instantiated (but its child classes might be).
/// Binds to an always-throwing allocator.
/// (This can also be used on final non-abstract classes that you just want to disable `new` for.
///  We just didn't find a better name for this concept than "abstract")
const METADATA_ABSTRACT: &str = "Abstract";
/// A slot defined by an AS3 `const` or `var` but that should have its slot ID
/// recorded so that it can be directly accessed by native code.
const METADATA_NATIVE_ACCESSIBLE: &str = "NativeAccessible";
/// Like `METADATA_NATIVE_ACCESSIBLE`, but for methods instead of slots.
const METADATA_NATIVE_CALLABLE: &str = "NativeCallable";
/// Indicates that this method does not read any properties of the Activation
/// passed to it except UpdateContext fields. This is used as an optimization.
const METADATA_FAST_CALL: &str = "FastCall";
// The name for metadata for namespace versioning- the Flex SDK doesn't
// strip versioning metadata, so we have to allow this metadata name
const API_METADATA_NAME: &str = "API";

/// If successful, returns a list of paths that were used. If this is run
/// from a build script, these paths should be printed with
/// cargo:rerun-if-changed
pub fn build_playerglobal(
    repo_root: PathBuf,
    out_dir: PathBuf,
    with_stubs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let classes_dir = repo_root.join("core/src/avm2/globals/");
    let asc_path = repo_root.join("core/build_playerglobal/asc.jar");

    let out_path = out_dir.join("playerglobal.swf");

    // This will create 'playerglobal.abc', 'playerglobal.cpp', and 'playerglobal.h'
    // in `out_dir`
    let status = Command::new("java")
        .args([
            "-classpath",
            &asc_path.to_string_lossy(),
            "macromedia.asc.embedding.ScriptCompiler",
            "-optimize",
            "-builtin",
            "-apiversioning",
            "-version",
            "9",
            "-outdir",
            &out_dir.to_string_lossy(),
            "-out",
            "playerglobal",
            &classes_dir.join("Toplevel.as").to_string_lossy(),
            &classes_dir.join("globals.as").to_string_lossy(),
        ])
        .status();
    match status {
        Ok(code) => {
            if !code.success() {
                return Err(format!("Compiling failed with code {code:?}").into());
            }
        }
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                return Err("Java could not be found on your computer. Please install java, then try compiling again.".into());
            }
            return Err(err.into());
        }
    }

    let playerglobal = out_dir.join("playerglobal");
    let mut bytes = std::fs::read(playerglobal.with_extension("abc"))?;

    // Cleanup the temporary files written out by 'asc.jar'
    std::fs::remove_file(playerglobal.with_extension("abc"))?;
    std::fs::remove_file(playerglobal.with_extension("cpp"))?;
    std::fs::remove_file(playerglobal.with_extension("h"))?;

    if with_stubs {
        collect_stubs(&classes_dir, &out_dir)?;
    }

    bytes = write_native_table(&bytes, &out_dir)?;

    let tags = [Tag::DoAbc2(DoAbc2 {
        flags: DoAbc2Flag::LAZY_INITIALIZE,
        name: "".into(),
        data: &bytes,
    })];

    let header = Header::default_with_swf_version(19);
    let out_file = File::create(out_path).unwrap();
    swf::write_swf(&header, &tags, out_file)?;

    Ok(())
}

// Resolve the 'name' field of a `Multiname`. This only handles the cases
// that we need for our custom `playerglobal.swf` (
fn resolve_multiname_name<'a>(abc: &'a AbcFile, multiname: &Multiname) -> Cow<'a, str> {
    if let Multiname::QName { name, .. } | Multiname::Multiname { name, .. } = multiname {
        String::from_utf8_lossy(&abc.constant_pool.strings[name.0 as usize - 1])
    } else {
        panic!("Unexpected Multiname {multiname:?}");
    }
}

// Strips off the version mark inserted by 'asc.jar',
// giving us a valid Rust module name. The actual versioning logic
// is handling in Ruffle when we load playerglobals
fn strip_version_mark(val: Cow<'_, str>) -> Cow<'_, str> {
    const MIN_API_MARK: usize = 0xE000;
    const MAX_API_MARK: usize = 0xF8FF;

    if let Some(chr) = val.chars().last()
        && chr as usize >= MIN_API_MARK
        && chr as usize <= MAX_API_MARK
    {
        // The version mark is 3 bytes in utf-8
        return val[..val.len() - 3].to_string().into();
    }
    val
}

// Like `resolve_multiname_name`, but for namespaces instead.
fn resolve_multiname_ns<'a>(abc: &'a AbcFile, multiname: &Multiname) -> Cow<'a, str> {
    let ns = match multiname {
        Multiname::QName { namespace, .. } => {
            &abc.constant_pool.namespaces[namespace.0 as usize - 1]
        }
        Multiname::Multiname { namespace_set, .. } => {
            if namespace_set.0 == 0 {
                panic!("Multiname namespace set must not be null");
            }

            let actual_index = namespace_set.0 as usize - 1;
            let ns_set = abc
                .constant_pool
                .namespace_sets
                .get(actual_index)
                .unwrap_or_else(|| panic!("Unknown namespace set constant {actual_index}"));

            if ns_set.len() == 1 {
                &abc.constant_pool.namespaces[ns_set[0].0 as usize - 1]
            } else {
                panic!("Found multiple namespaces in namespace set {ns_set:?}")
            }
        }
        _ => panic!("Unexpected Multiname {multiname:?}"),
    };
    let namespace = if let Namespace::Package(p) | Namespace::PackageInternal(p) = ns {
        &abc.constant_pool.strings[p.0 as usize - 1]
    } else {
        panic!("Unexpected Namespace {ns:?}");
    };
    strip_version_mark(String::from_utf8_lossy(namespace))
}

fn flash_to_rust_string(path: &str, uppercase: bool, separator: &str) -> String {
    // For the specific case of the package name __AS3__.vec, we pretend that the
    // namespace was empty. Otherwise, we'd need to define vector classes in the
    // folder `avm2/globals/__as3__/vec/`.
    if path == "__AS3__.vec" {
        return "".to_string();
    }

    let new_case = if uppercase {
        Case::UpperSnake
    } else {
        Case::Snake
    };

    // Convert each component of the path to snake-case.
    // This correctly handles sequences of upper-case letters,
    // so 'URLLoader' becomes 'url_loader'
    let components = path
        .split('.')
        .map(|component| {
            // Special-case this so that it matches the Flash namespace
            if component == "display3D" {
                return component.to_string();
            }

            let mut remove_boundaries = vec![Boundary::DigitUpper];
            // Special case for classes ending in '3D' - we want to have something like
            // 'vertex_buffer_3d' instead of 'vertex_buffer3d'
            if !component.ends_with("3D") {
                // Do not split on a letter followed by a digit, so e.g. `atan2` won't become `atan_2`.
                remove_boundaries.extend([Boundary::UpperDigit, Boundary::LowerDigit]);
            }

            // For cases like `Vector$int`, so we don't have to put the native
            // methods in a file with a '$' in its name
            let component = component.replace('$', "_");

            component
                .from_case(Case::Camel)
                .remove_boundaries(&remove_boundaries)
                .to_case(new_case)
        })
        .collect::<Vec<_>>();

    // Form a Rust path from the components
    components.join(separator)
}

fn rust_method_path(
    abc: &AbcFile,
    trait_: &Trait,
    parent: Option<Index<Multiname>>,
    prefix: &str,
    suffix: &str,
) -> TokenStream {
    let mut path = "crate::avm2::globals::".to_string();

    let trait_name = &abc.constant_pool.multinames[trait_.name.0 as usize - 1];

    if let Some(parent) = parent {
        // This is a method defined inside the class. Append the class namespace
        // (the package) and the class name.
        // For example, a namespace of "flash.system" and a name of "Security"
        // turns into the path "flash::system::security"
        let multiname = &abc.constant_pool.multinames[parent.0 as usize - 1];
        let ns = flash_to_rust_string(&resolve_multiname_ns(abc, multiname), false, "::");
        if !ns.is_empty() {
            path += &ns;
            path += "::";
        }
        let name = resolve_multiname_name(abc, multiname);
        path += &flash_to_rust_string(&name, false, "::");
        path += "::";
    } else {
        // This is a freestanding function. Append its namespace (the package).
        // For example, the freestanding function "flash.utils.getDefinitionByName"
        // has a namespace of "flash.utils", which turns into the path
        // "flash::utils"
        let name = resolve_multiname_ns(abc, trait_name);
        let ns = &flash_to_rust_string(&name, false, "::");
        path += ns;
        if !ns.is_empty() {
            path += "::";
        }
    }

    // Append the trait name - this corresponds to the actual method
    // name (e.g. `getDefinitionByName`)
    path += prefix;

    let name = resolve_multiname_name(abc, trait_name).to_string();

    path += &flash_to_rust_string(&name, false, "::");

    path += suffix;

    // Now that we've built up the path, convert it into a `TokenStream`.
    // This gives us something like
    // `crate::avm2::globals::flash::system::Security::allowDomain`
    //
    // The resulting `TokenStream` is suitable for usage with `quote!` to
    // generate a reference to the function pointer that should exist
    // at that path in Rust code.
    let path_tokens = TokenStream::from_str(&path).unwrap();
    quote! { Some(#path_tokens) }
}

fn rust_path_and_trait_name(
    abc: &AbcFile,
    trait_: &Trait,
    parent: Option<Index<Multiname>>,
) -> (String, String) {
    let mut path = String::new();

    let trait_name = &abc.constant_pool.multinames[trait_.name.0 as usize - 1];

    if let Some(parent) = parent {
        // This is a method defined inside the class. Append the class namespace
        // (the package) and the class name.
        // For example, a namespace of "flash.system" and a name of "Security"
        // turns into the path "flash_system_security"
        let multiname = &abc.constant_pool.multinames[parent.0 as usize - 1];
        let ns = flash_to_rust_string(&resolve_multiname_ns(abc, multiname), false, "_");
        if !ns.is_empty() {
            path += &ns;
            path += "_";
        }
        let name = resolve_multiname_name(abc, multiname);

        path += &flash_to_rust_string(&name, false, "_");
    } else {
        panic!("Freestanding traits not supported");
    }

    let name = resolve_multiname_name(abc, trait_name);
    let name = flash_to_rust_string(&name, true, "_");

    (path, name)
}

fn strip_metadata(abc: &mut AbcFile) {
    abc.metadata.clear();
    for instance in &mut abc.instances {
        for trait_ in &mut instance.traits {
            trait_.metadata.clear();
        }
    }
    for class in &mut abc.classes {
        for trait_ in &mut class.traits {
            trait_.metadata.clear();
        }
    }
    for script in &mut abc.scripts {
        for trait_ in &mut script.traits {
            trait_.metadata.clear();
        }
    }
    for body in &mut abc.method_bodies {
        for trait_ in &mut body.traits {
            trait_.metadata.clear();
        }
    }
}

/// If we don't properly declare 'namespace AS3' in the input to asc.jar, then
/// a call like `self.AS3::toXMLString()` will end up getting compiled to weird bytecode like this:
///
/// ```pcode
/// getlex Multiname("AS3",[PackageNamespace(""),PrivateNamespace(null,"35"),PackageInternalNs(""),PrivateNamespace(null,"33"),ProtectedNamespace("XML"),StaticProtectedNs("XML")])
/// coerce QName(PackageNamespace(""),"Namespace")
/// getproperty RTQName("toXMLString")
/// getlocal2
/// call 0
/// ```
///
/// This will cause a new bound method to be created, instead of going through 'callproperty'.
///
/// We detect this case by looking for the weird 'getlex AS3', which should never happen normally.
fn check_weird_namespace_lookup(abc: &AbcFile) -> Result<(), Box<dyn std::error::Error>> {
    for body in &abc.method_bodies {
        let mut reader = Reader::new(&body.code);
        while reader.pos(&body.code) != body.code.len() {
            let op: Op = reader.read_op()?;
            if let Op::GetLex { index } = op {
                let multiname = &abc.constant_pool.multinames[index.0 as usize - 1];
                if let Multiname::QName { name, .. } | Multiname::Multiname { name, .. } = multiname
                {
                    let name =
                        String::from_utf8_lossy(&abc.constant_pool.strings[name.0 as usize - 1]);
                    if name == "AS3" {
                        panic!(
                            r#"Found getlex of "AS3" in method body. Make sure you have `namespace AS3 = "http://adobe.com/AS3/2006/builtin";` in your `package` block"#
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// Checks if a trait has a certain metadata on it.
fn trait_has_metadata(abc: &AbcFile, trait_: &Trait, metadata_value: &str) -> bool {
    for metadata_idx in &trait_.metadata {
        let metadata = &abc.metadata[metadata_idx.0 as usize];
        let name =
            String::from_utf8_lossy(&abc.constant_pool.strings[metadata.name.0 as usize - 1]);

        let is_versioning = match &*name {
            RUFFLE_METADATA_NAME => false,
            API_METADATA_NAME => true,
            _ => panic!("Unexpected class metadata {name:?}"),
        };

        for item in &metadata.items {
            let key = if item.key.0 != 0 {
                Some(&abc.constant_pool.strings[item.key.0 as usize - 1])
            } else {
                None
            };
            let value =
                String::from_utf8_lossy(&abc.constant_pool.strings[item.value.0 as usize - 1]);

            if key.is_none() && &*value == metadata_value && !is_versioning {
                return true;
            }
        }
    }

    false
}

/// Handles native functions defined in our `playerglobal`
///
/// The high-level idea is to generate code (specifically, a `TokenStream`)
/// which builds a table - mapping from the method ids of native functions,
/// to Rust function pointers which implement them.
///
/// This table gets used when we first load a method from an ABC file.
/// If it's a native method in our `playerglobal`, we swap it out
/// with a `NativeMethod` retrieved from the table. To the rest of
/// the Ruffle codebase, it appears as though the method was always defined
/// as a native method, and never existed in the bytecode at all.
///
/// See `flash.system.Security.allowDomain` for an example of defining
/// and using a native method.
///
/// Returns a modified version of 'data' that should be saved to disk
/// in our generated SWF
fn write_native_table(data: &[u8], out_dir: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut reader = swf::avm2::read::Reader::new(data);
    let mut abc = reader.read()?;

    check_weird_namespace_lookup(&abc)?;

    let none_tokens = quote! { None };
    let mut rust_paths = vec![none_tokens.clone(); abc.methods.len()];
    let mut rust_instance_allocators = vec![none_tokens.clone(); abc.classes.len()];
    let mut rust_call_handlers = vec![none_tokens.clone(); abc.classes.len()];
    let mut rust_custom_constructors = vec![none_tokens; abc.classes.len()];
    let mut rust_fast_calls = vec![];

    let mut rust_accessible_slots: HashMap<String, Vec<_>> = HashMap::new();
    let mut rust_accessible_methods: HashMap<String, Vec<_>> = HashMap::new();

    let mut check_trait = |trait_: &Trait, parent: Option<Index<Multiname>>, is_class: bool| {
        match trait_.kind {
            TraitKind::Slot { slot_id, .. } | TraitKind::Const { slot_id, .. } => {
                if trait_has_metadata(&abc, trait_, METADATA_NATIVE_ACCESSIBLE) {
                    if slot_id == 0 {
                        panic!(
                            "ASC should calculate slot ids for all slots; cannot apply NativeAccessible without a compiler-calculated slot id"
                        )
                    } else {
                        // Slots are 1-indexed!
                        let slot_id = slot_id - 1;

                        let (trait_name, const_name) =
                            rust_path_and_trait_name(&abc, trait_, parent);
                        let const_name = quote::format_ident!("{}", const_name);

                        // Declare a const with the slot name set to the slot id.
                        rust_accessible_slots
                            .entry(trait_name)
                            .or_default()
                            .push(quote! {
                                pub const #const_name: u32 = #slot_id;
                            });
                    }
                }
            }
            TraitKind::Method { disp_id, method }
            | TraitKind::Getter { disp_id, method }
            | TraitKind::Setter { disp_id, method } => {
                if trait_has_metadata(&abc, trait_, METADATA_NATIVE_CALLABLE) {
                    if disp_id == 0 {
                        panic!(
                            "ASC should calculate disp ids for all methods; cannot apply NativeCallable without a compiler-calculated disp id"
                        )
                    } else {
                        // Disp-ids are 1-indexed, but ASC generates them two disp-ids
                        // too low for class methods and one disp-id too high for
                        // instance methods. Instead of subtracting 1 from it the disp-id,
                        // add 1 if it's a class method, or subtract 2 if it's
                        // an instance method.
                        let disp_id = if is_class { disp_id + 1 } else { disp_id - 2 };

                        let (trait_name, const_name) =
                            rust_path_and_trait_name(&abc, trait_, parent);

                        // Make getters prefixed with GET_ and setters prefixed with SET_
                        let const_name = match trait_.kind {
                            TraitKind::Getter { .. } => quote::format_ident!("GET_{}", const_name),
                            TraitKind::Setter { .. } => quote::format_ident!("SET_{}", const_name),
                            _ => quote::format_ident!("{}", const_name),
                        };

                        // Declare a const with the method name set to the disp id.
                        rust_accessible_methods
                            .entry(trait_name)
                            .or_default()
                            .push(quote! {
                                pub const #const_name: u32 = #disp_id;
                            });
                    }
                }

                let method_id = method.0 as usize;

                let abc_method = &abc.methods[method_id];
                // We only want to process native methods
                if !abc_method.flags.contains(MethodFlags::NATIVE) {
                    return;
                }

                if trait_has_metadata(&abc, trait_, METADATA_FAST_CALL) {
                    rust_fast_calls.push(quote! { #method_id });
                }

                // Note - technically, this could conflict with
                // a method with a name starting with `get_` or `set_`.
                // However, all Flash methods are named with lowerCamelCase,
                // so we'll never actually need to implement a native method that
                // would cause such a conflict.
                let method_prefix = match trait_.kind {
                    TraitKind::Getter { .. } => "get_",
                    TraitKind::Setter { .. } => "set_",
                    _ => "",
                };

                rust_paths[method_id] = rust_method_path(&abc, trait_, parent, method_prefix, "");
            }
            TraitKind::Function { .. } => {
                panic!("TraitKind::Function is not supported: {trait_:?}")
            }
            _ => {}
        }
    };

    // We support four kinds of native methods:
    // instance methods, class methods, script methods ("freestanding") and initializers.
    // We're going to insert them into an array indexed by `MethodId`,
    // so it doesn't matter what order we visit them in.
    for (i, instance) in abc.instances.iter().enumerate() {
        // Look for native instance methods
        for trait_ in &instance.traits {
            check_trait(trait_, Some(instance.name), false);
        }
        // Look for native class methods (in the corresponding
        // `Class` definition)
        for trait_ in &abc.classes[i].traits {
            check_trait(trait_, Some(instance.name), true);
        }
    }
    // Look for freestanding methods
    for script in &abc.scripts {
        for trait_ in &script.traits {
            check_trait(trait_, None, false);
        }
    }

    // Look for `[Ruffle(InstanceAllocator)]` and similar metadata - if present,
    // generate a reference to a function in the native instance
    // allocators table.
    let mut check_class = |trait_: &Trait| {
        let class_id = if let TraitKind::Class { class, .. } = trait_.kind {
            class.0
        } else {
            return;
        };

        let class_name_idx = abc.instances[class_id as usize].name;
        let class_name = resolve_multiname_name(
            &abc,
            &abc.constant_pool.multinames[class_name_idx.0 as usize - 1],
        );

        let instance_allocator_method_name =
            "::".to_string() + &flash_to_rust_string(&class_name, false, "::") + "_allocator";
        let init_method_name =
            "::".to_string() + &flash_to_rust_string(&class_name, false, "::") + "_initializer";
        let call_handler_method_name = "::call_handler".to_string();
        let custom_constructor_method_name =
            "::".to_string() + &flash_to_rust_string(&class_name, false, "::") + "_constructor";

        // Also support instance initializer - let's pretend it's a trait.
        let init_method_idx = abc.instances[class_id as usize].init_method;
        let init_method = &abc.methods[init_method_idx.0 as usize];
        if init_method.flags.contains(MethodFlags::NATIVE) {
            let init_trait = Trait {
                name: class_name_idx,
                kind: TraitKind::Method {
                    disp_id: 0, // unused
                    method: abc.classes[class_id as usize].init_method,
                },
                metadata: vec![],   // unused
                is_final: true,     // unused
                is_override: false, // unused
            };
            rust_paths[init_method_idx.0 as usize] =
                rust_method_path(&abc, &init_trait, None, "", &init_method_name);
        }

        for metadata_idx in &trait_.metadata {
            let metadata = &abc.metadata[metadata_idx.0 as usize];
            let name =
                String::from_utf8_lossy(&abc.constant_pool.strings[metadata.name.0 as usize - 1]);

            let is_versioning = match &*name {
                RUFFLE_METADATA_NAME => false,
                API_METADATA_NAME => true,
                _ => panic!("Unexpected class metadata {name:?}"),
            };

            for item in &metadata.items {
                let key = if item.key.0 != 0 {
                    Some(&abc.constant_pool.strings[item.key.0 as usize - 1])
                } else {
                    None
                };
                let value =
                    String::from_utf8_lossy(&abc.constant_pool.strings[item.value.0 as usize - 1]);
                match (key, &*value) {
                    // Match `[Ruffle(InstanceAllocator)]`
                    (None, METADATA_INSTANCE_ALLOCATOR) if !is_versioning => {
                        // This results in a path of the form
                        // `crate::avm2::globals::<path::to::class>::<class_allocator>`
                        rust_instance_allocators[class_id as usize] = rust_method_path(
                            &abc,
                            trait_,
                            None,
                            "",
                            &instance_allocator_method_name,
                        );
                    }
                    (None, METADATA_CALL_HANDLER) if !is_versioning => {
                        rust_call_handlers[class_id as usize] =
                            rust_method_path(&abc, trait_, None, "", &call_handler_method_name);
                    }
                    (None, METADATA_CUSTOM_CONSTRUCTOR) if !is_versioning => {
                        rust_custom_constructors[class_id as usize] = rust_method_path(
                            &abc,
                            trait_,
                            None,
                            "",
                            &custom_constructor_method_name,
                        );
                    }
                    (None, METADATA_ABSTRACT) if !is_versioning => {
                        rust_instance_allocators[class_id as usize] = {
                            let path = "crate::avm2::object::abstract_class_allocator";
                            let path_tokens = TokenStream::from_str(path).unwrap();
                            quote! { Some(#path_tokens) }
                        };
                    }
                    (None, METADATA_CONSTRUCT_ON_CALL) if !is_versioning => {
                        rust_call_handlers[class_id as usize] = {
                            let path = "crate::avm2::object::construct_call_handler";
                            let path_tokens = TokenStream::from_str(path).unwrap();
                            quote! { Some(#path_tokens) }
                        };
                    }
                    (None, _) if is_versioning => {}
                    _ => panic!("Unexpected metadata pair ({key:?}, {value})"),
                }
            }
        }
    };

    // Handle classes
    for script in &abc.scripts {
        for trait_ in &script.traits {
            check_class(trait_);
        }
    }

    // Finally, generate the actual code.
    let rust_accessible_slots = rust_accessible_slots.into_iter().map(|(mod_name, consts)| {
        let mod_name = quote::format_ident!("{}", mod_name);
        quote! {
            pub mod #mod_name {
                #(#consts)*
            }
        }
    });

    let rust_accessible_methods = rust_accessible_methods
        .into_iter()
        .map(|(mod_name, consts)| {
            let mod_name = quote::format_ident!("{}", mod_name);
            quote! {
                pub mod #mod_name {
                    #(#consts)*
                }
            }
        });

    let make_native_table = quote! {
        // This is a Rust array -
        // the entry at index `i` is the method name and Rust function pointer for the native
        // method with id `i`. Not all methods in playerglobal will be native
        // methods, so we store `None` in the entries corresponding to non-native
        // functions. We expect the majority of the methods in playerglobal to be
        // native, so this should only waste a small amount of memory.
        //
        // If a function pointer doesn't exist at the expected path,
        // then Ruffle compilation will fail
        // with an error message that mentions the non-existent path.
        //
        // When we initially load a method from an ABC file, we check if it's from our playerglobal,
        // and if its ID exists in this table.
        // If so, we replace it with a `NativeMethod` constructed
        // from the function pointer we looked up in the table.
        pub const NATIVE_METHOD_TABLE: &[Option<crate::avm2::method::NativeMethodImpl>] = &[
            #(#rust_paths,)*
        ];

        // This is very similar to `NATIVE_METHOD_TABLE`, but we have one entry per
        // class, rather than per method. When an entry is `Some(fn_ptr)`, we use
        // `fn_ptr` as the instance allocator for the corresponding class when we
        // load it into Ruffle.
        pub const NATIVE_INSTANCE_ALLOCATOR_TABLE: &[Option<crate::avm2::class::AllocatorFn>] = &[
            #(#rust_instance_allocators,)*
        ];

        // This is very similar to `NATIVE_INSTANCE_ALLOCATOR_TABLE`.
        // When an entry is `Some(fn_ptr)`, we use `fn_ptr` as the native call
        // handler for the corresponding class when we load it into Ruffle.
        pub const NATIVE_CALL_HANDLER_TABLE: &[Option<crate::avm2::method::NativeMethodImpl>] = &[
            #(#rust_call_handlers,)*
        ];

        // This is very similar to `NATIVE_INSTANCE_ALLOCATOR_TABLE`.
        // When an entry is `Some(fn_ptr)`, we use `fn_ptr` as the native custom
        // constructor for the corresponding class when we load it into Ruffle.
        pub const NATIVE_CUSTOM_CONSTRUCTOR_TABLE: &[Option<crate::avm2::class::CustomConstructorFn>] = &[
            #(#rust_custom_constructors,)*
        ];

        // This is an array containing the method ids of every method marked
        // "[Ruffle(FastCall)]". Unlike the rest, it is not indexed by method id-
        // instead, every item in the list is a method id.
        //
        // FIXME: should this be some sort of hashset?
        pub const NATIVE_FAST_CALL_LIST: &[usize] = &[
            #(#rust_fast_calls,)*
        ];

        pub mod slots {
            #(#rust_accessible_slots)*
        }

        pub mod methods {
            #(#rust_accessible_methods)*
        }
    }
    .to_string();

    // Each table entry ends with ') ,' - insert a newline so that
    // each entry is on its own line. This makes error messages more readable.
    // Also, internal slot definitions end with '; const'. Insert a newline
    // for them too.
    let make_native_table = make_native_table
        .replace(") ,", ") ,\n")
        .replace("; const", ";\nconst");

    let mut native_table_file = File::create(out_dir.join("native_table.rs"))?;
    native_table_file.write_all(make_native_table.as_bytes())?;

    // Ruffle doesn't need metadata items at runtime, so strip
    // them out to save space
    strip_metadata(&mut abc);

    let mut out_bytes = Vec::new();
    let mut writer = Writer::new(&mut out_bytes);
    writer.write(abc).expect("Failed to write modified ABC");

    Ok(out_bytes)
}

fn collect_stubs(root: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let pattern = RegexBuilder::new(
        r#"
            \b (?P<type> stub_method | stub_getter | stub_setter | stub_constructor) \s*
            \( \s*
                "(?P<class> .+)" \s*
                , \s*
                "(?P<property> .+)" \s*
                (?:|
                    , \s*
                    "(?P<specifics> .+)" \s*
                )
            \) \s*
            ;
        "#,
    )
    .ignore_whitespace(true)
    .build()?;

    let mut stubs = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|f| f.ok())
        .filter(|f| f.path().extension() == Some(OsStr::new("as")))
    {
        let contents = fs::read_to_string(entry.path())?;
        for entry in pattern.captures_iter(&contents) {
            let class = &entry["class"];
            let property = entry.name("property").map(|m| m.as_str());
            let specifics = entry.name("specifics").map(|m| m.as_str());

            match (&entry["type"], property, specifics) {
                ("stub_method", Some(property), Some(specifics)) => stubs.push(quote! {
                    crate::stub::Stub::Avm2Method {
                        class: Cow::Borrowed(#class),
                        method: Cow::Borrowed(#property),
                        specifics: Cow::Borrowed(#specifics)
                    }
                }),
                ("stub_method", Some(property), None) => stubs.push(quote! {
                    crate::stub::Stub::Avm2Method {
                        class: Cow::Borrowed(#class),
                        method: Cow::Borrowed(#property),
                        specifics: None
                    }
                }),
                ("stub_getter", Some(property), _) => stubs.push(quote! {
                    crate::stub::Stub::Avm2Getter {
                        class: Cow::Borrowed(#class),
                        property: Cow::Borrowed(#property)
                    }
                }),
                ("stub_setter", Some(property), _) => stubs.push(quote! {
                    crate::stub::Stub::Avm2Setter {
                        class: Cow::Borrowed(#class),
                        property: Cow::Borrowed(#property)
                    }
                }),
                ("stub_constructor", Some(property), _) => stubs.push(quote! {
                    // Property is actually specifics here
                    crate::stub::Stub::Avm2Constructor {
                        class: Cow::Borrowed(#class),
                        specifics: Some(Cow::Borrowed(#property))
                    }
                }),
                ("stub_constructor", None, _) => stubs.push(quote! {
                    crate::stub::Stub::Avm2Constructor {
                        class: Cow::Borrowed(#class),
                        specifics: None
                    }
                }),
                _ => panic!("Unsupported stub type {}", &entry["type"]),
            }
        }
    }

    let stub_block = quote! {
        #[cfg(feature = "known_stubs")]
        use std::borrow::Cow;

        #[cfg(feature = "known_stubs")]
        pub static AS_DEFINED_STUBS: &[crate::stub::Stub] = &[
            #(#stubs,)*
        ];
    };

    let mut as_stub_file = File::create(out_dir.join("actionscript_stubs.rs"))?;
    as_stub_file.write_all(stub_block.to_string().as_bytes())?;

    Ok(())
}
