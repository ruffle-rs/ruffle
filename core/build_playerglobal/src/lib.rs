//! An internal Ruffle utility to build our playerglobal
//! `library.swf`

use convert_case::{Boundary, Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use regex::RegexBuilder;
use std::borrow::Cow;
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
// Indicates that we should generate a reference to a native initializer
// method (used as a metadata key with `Ruffle` metadata)
const METADATA_SUPER_INITIALIZER: &str = "SuperInitializer";
/// Indicates that we should generate a reference to a class call handler
/// method (used as a metadata key with `Ruffle` metadata)
const METADATA_CALL_HANDLER: &str = "CallHandler";
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
            "-import",
            &classes_dir.join("stubs.as").to_string_lossy(),
            // From some reason this has to be passed as a separate argument.
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

    if let Some(chr) = val.chars().last() {
        if chr as usize >= MIN_API_MARK && chr as usize <= MAX_API_MARK {
            // The version mark is 3 bytes in utf-8
            return val[..val.len() - 3].to_string().into();
        }
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
                .unwrap_or_else(|| panic!("Unknown namespace set constant {}", actual_index));

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

fn flash_to_rust_path(path: &str) -> String {
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

            let mut without_boundaries = vec![Boundary::DigitUpper];
            // Special case for classes ending in '3D' - we want to ave something like
            // 'vertex_buffer_3d' instead of 'vertex_buffer3d'
            if !component.ends_with("3D") {
                // Do not split on a letter followed by a digit, so e.g. `atan2` won't become `atan_2`.
                without_boundaries.extend([Boundary::UpperDigit, Boundary::LowerDigit]);
            }

            component
                .from_case(Case::Camel)
                .without_boundaries(&without_boundaries)
                .to_case(Case::Snake)
        })
        .collect::<Vec<_>>();
    // Form a Rust path from the snake-case components
    components.join("::")
}

fn rust_method_name_and_path(
    abc: &AbcFile,
    trait_: &Trait,
    parent: Option<Index<Multiname>>,
    prefix: &str,
    suffix: &str,
) -> TokenStream {
    let mut path = "crate::avm2::globals::".to_string();

    let trait_name = &abc.constant_pool.multinames[trait_.name.0 as usize - 1];
    // We build up the Flash method path (e.g. flash.utils::getDefinitionByName)
    // and store it in the table. This gets used by Ruffle to display proper
    // stack traces involving native methods.
    let mut flash_method_path = String::new();

    if let Some(parent) = parent {
        // This is a method defined inside the class. Append the class namespace
        // (the package) and the class name.
        // For example, a namespace of "flash.system" and a name of "Security"
        // turns into the path "flash::system::security"
        let multiname = &abc.constant_pool.multinames[parent.0 as usize - 1];
        let ns = flash_to_rust_path(&resolve_multiname_ns(abc, multiname));
        if !ns.is_empty() {
            path += &ns;
            path += "::";

            flash_method_path += &ns;
            flash_method_path += "::";
        }
        let name = resolve_multiname_name(abc, multiname);
        path += &flash_to_rust_path(&name);
        path += "::";

        flash_method_path += &*name;
        flash_method_path += "::";
    } else {
        // This is a freestanding function. Append its namespace (the package).
        // For example, the freestanding function "flash.utils.getDefinitionByName"
        // has a namespace of "flash.utils", which turns into the path
        // "flash::utils"
        let name = resolve_multiname_ns(abc, trait_name);
        let ns = &flash_to_rust_path(&name);
        path += ns;
        flash_method_path += &name;
        if !ns.is_empty() {
            path += "::";
            flash_method_path += "::";
        }
    }

    // Append the trait name - this corresponds to the actual method
    // name (e.g. `getDefinitionByName`)
    path += prefix;

    let name = resolve_multiname_name(abc, trait_name).to_string();

    path += &flash_to_rust_path(&name);
    flash_method_path += &name;

    path += suffix;

    // Now that we've built up the path, convert it into a `TokenStream`.
    // This gives us something like
    // `crate::avm2::globals::flash::system::Security::allowDomain`
    //
    // The resulting `TokenStream` is suitable for usage with `quote!` to
    // generate a reference to the function pointer that should exist
    // at that path in Rust code.
    let path_tokens = TokenStream::from_str(&path).unwrap();
    quote! { Some((#flash_method_path, #path_tokens)) }
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
    let mut rust_super_initializers = vec![none_tokens.clone(); abc.classes.len()];
    let mut rust_call_handlers = vec![none_tokens; abc.classes.len()];

    let mut check_trait = |trait_: &Trait, parent: Option<Index<Multiname>>| {
        let method_id = match trait_.kind {
            TraitKind::Method { method, .. }
            | TraitKind::Getter { method, .. }
            | TraitKind::Setter { method, .. } => {
                let abc_method = &abc.methods[method.0 as usize];
                // We only want to process native methods
                if !abc_method.flags.contains(MethodFlags::NATIVE) {
                    return;
                }
                method
            }
            TraitKind::Function { .. } => {
                panic!("TraitKind::Function is not supported: {trait_:?}")
            }
            _ => return,
        };

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

        rust_paths[method_id.0 as usize] =
            rust_method_name_and_path(&abc, trait_, parent, method_prefix, "");
    };

    // Look for `[Ruffle(InstanceAllocator)]` metadata - if present,
    // generate a reference to an allocator function in the native instance
    // allocators table.
    let mut check_instance_allocator = |trait_: &Trait| {
        let class_id = if let TraitKind::Class { class, .. } = trait_.kind {
            class.0
        } else {
            return;
        };

        let class_name_idx = abc.instances[class_id as usize].name.0;
        let class_name = resolve_multiname_name(
            &abc,
            &abc.constant_pool.multinames[class_name_idx as usize - 1],
        );

        let instance_allocator_method_name =
            "::".to_string() + &flash_to_rust_path(&class_name) + "_allocator";
        let super_init_method_name = "::super_init".to_string();
        let call_handler_method_name = "::call_handler".to_string();
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
                        rust_instance_allocators[class_id as usize] = rust_method_name_and_path(
                            &abc,
                            trait_,
                            None,
                            "",
                            &instance_allocator_method_name,
                        );
                    }
                    (None, METADATA_SUPER_INITIALIZER) if !is_versioning => {
                        rust_super_initializers[class_id as usize] = rust_method_name_and_path(
                            &abc,
                            trait_,
                            None,
                            "",
                            &super_init_method_name,
                        )
                    }
                    (None, METADATA_CALL_HANDLER) if !is_versioning => {
                        rust_call_handlers[class_id as usize] = rust_method_name_and_path(
                            &abc,
                            trait_,
                            None,
                            "",
                            &call_handler_method_name,
                        )
                    }
                    (None, _) if is_versioning => {}
                    _ => panic!("Unexpected metadata pair ({key:?}, {value})"),
                }
            }
        }
    };

    // We support three kinds of native methods:
    // instance methods, class methods, and freestanding functions.
    // We're going to insert them into an array indexed by `MethodId`,
    // so it doesn't matter what order we visit them in.
    for (i, instance) in abc.instances.iter().enumerate() {
        // Look for native instance methods
        for trait_ in &instance.traits {
            check_trait(trait_, Some(instance.name));
        }
        // Look for native class methods (in the corresponding
        // `Class` definition)
        for trait_ in &abc.classes[i].traits {
            check_trait(trait_, Some(instance.name));
        }
    }

    // Look for freestanding methods
    for script in &abc.scripts {
        for trait_ in &script.traits {
            check_trait(trait_, None);
            check_instance_allocator(trait_);
        }
    }
    // Finally, generate the actual code.
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
        pub const NATIVE_METHOD_TABLE: &[Option<(&'static str, crate::avm2::method::NativeMethodImpl)>] = &[
            #(#rust_paths,)*
        ];

        // This is very similar to `NATIVE_METHOD_TABLE`, but we have one entry per
        // class, rather than per method. When an entry is `Some(fn_ptr)`, we use
        // `fn_ptr` as the instance allocator for the corresponding class when we
        // load it into Ruffle.
        pub const NATIVE_INSTANCE_ALLOCATOR_TABLE: &[Option<(&'static str, crate::avm2::class::AllocatorFn)>] = &[
            #(#rust_instance_allocators,)*
        ];

        // This is very similar to `NATIVE_METHOD_TABLE`, but we have one entry per
        // class, rather than per method. When an entry is `Some(fn_ptr)`, we use
        // `fn_ptr` as the super initializer for the corresponding class when we
        // load it into Ruffle.
        pub const NATIVE_SUPER_INITIALIZER_TABLE: &[Option<(&'static str, crate::avm2::method::NativeMethodImpl)>] = &[
            #(#rust_super_initializers,)*
        ];

        // This is very similar to `NATIVE_SUPER_INITIALIZER_TABLE`.
        // When an entry is `Some(fn_ptr)`, we use
        // `fn_ptr` as the native call handler for the corresponding class when we
        // load it into Ruffle.
        pub const NATIVE_CALL_HANDLER_TABLE: &[Option<(&'static str, crate::avm2::method::NativeMethodImpl)>] = &[
            #(#rust_call_handlers,)*
        ];
    }
    .to_string();

    // Each table entry ends with ') ,' - insert a newline so that
    // each entry is on its own line. This makes error messages more readable.
    let make_native_table = make_native_table.replace(") ,", ") ,\n");

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
