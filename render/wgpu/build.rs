// #[cfg(feature = "build_shaders")]
use naga;

fn print_error(e: &Box<dyn std::error::Error>) {
    eprint!("{}", e);

    let mut e = e.source();
    if e.is_some() {
        eprintln!(": ");
    } else {
        eprintln!();
    }

    while let Some(source) = e {
        eprintln!("\t{}", source);
        e = source.source();
    }
}

fn compile_shader(src: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse WGSL shader.
    let result = naga::front::wgsl::parse_str(&src);
    let module = result.map_err(|e| {
        // TODO: Use `emit_to_stderr_with_path` once it arrives.
        e.emit_to_stderr(&src);
        e
    })?;

    // Validate the IR.
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)?;

    // Output compiled SPIR-V shader.
    let options = naga::back::spv::Options::default();
    /*options.bounds_check_policies = params.bounds_check_policies;
    options.flags.set(
        spv::WriterFlags::ADJUST_COORDINATE_SPACE,
        !params.keep_coordinate_space,
    );*/
    let spv = naga::back::spv::write_vec(&module, &info, &options, None)?;
    let bytes = spv
        .iter()
        .fold(Vec::with_capacity(spv.len() * 4), |mut v, w| {
            v.extend_from_slice(&w.to_le_bytes());
            v
        });
    Ok(bytes)
}

// #[cfg(feature = "build_shaders")]
fn build_shader(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", "shaders/common.wgsl");
    const COMMON_SRC: &str = include_str!("shaders/common.wgsl");

    println!("cargo:rerun-if-changed={}", "shaders/output_linear.wgsl");
    const OUTPUT_LINEAR_SRC: &str = include_str!("shaders/output_linear.wgsl");

    println!("cargo:rerun-if-changed={}", "shaders/output_srgb.wgsl");
    const OUTPUT_SRGB_SRC: &str = include_str!("shaders/output_srgb.wgsl");

    let path = std::path::Path::new("shaders")
        .join(name)
        .with_extension("wgsl");
    println!("cargo:rerun-if-changed={}", path.display());
    let src = std::fs::read_to_string(path)?;

    let out_dir = std::env::var_os("OUT_DIR").ok_or("OUT_DIR not defined")?;
    let out_path = std::path::Path::new(&out_dir);

    let bytes = compile_shader(&[COMMON_SRC, OUTPUT_SRGB_SRC, &src].concat())?;
    std::fs::write(out_path.join(format!("{}_srgb.spv", name)), &bytes)?;

    let bytes = compile_shader(&[COMMON_SRC, OUTPUT_LINEAR_SRC, &src].concat())?;
    std::fs::write(out_path.join(format!("{}_linear.spv", name)), &bytes)?;

    Ok(())
}

fn build_shaders() -> Result<(), Box<dyn std::error::Error>> {
    build_shader("color")?;
    build_shader("bitmap")?;
    build_shader("gradient")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    build_shaders().map_err(|e| {
        print_error(&e);
        e
    })
}
