use naga::{
    valid::{Capabilities, ValidationFlags, Validator},
    Module,
};
use naga_agal::{agal_to_naga, VertexAttributeFormat};

pub fn to_wgsl(module: &Module) -> String {
    let mut out = String::new();

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let module_info = validator
        .validate(module)
        .unwrap_or_else(|e| panic!("Validation failed: {}", e));

    let mut writer =
        naga::back::wgsl::Writer::new(&mut out, naga::back::wgsl::WriterFlags::EXPLICIT_TYPES);

    writer.write(module, &module_info).expect("Writing failed");
    out
}

// Making this a macro gives us a better span in 'inta'
macro_rules! test_shader {
    ($shader:expr, $attrs:expr, $shader_type:expr $(,)?) => {
        let module = agal_to_naga(&$shader, $attrs).unwrap();
        let output = to_wgsl(&module);
        insta::assert_display_snapshot!(output);
    };
}

#[test]
fn test_shaders() {
    test_shader!(
        // m44 op, va0, vc0
        // mov v0, va1
        [
            160, 1, 0, 0, 0, 161, 0, 24, 0, 0, 0, 0, 0, 15, 3, 0, 0, 0, 228, 0, 0, 0, 0, 0, 0, 0,
            228, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 4, 1, 0, 0, 228, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0
        ],
        &[
            Some(VertexAttributeFormat::Float3),
            Some(VertexAttributeFormat::Float3),
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        ShaderType::Vertex,
    );

    test_shader!(
        // mov op, va0
        // mov v0, va1
        [
            160, 1, 0, 0, 0, 161, 0, 0, 0, 0, 0, 0, 0, 15, 3, 0, 0, 0, 228, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 15, 4, 1, 0, 0, 228, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ],
        &[
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        ShaderType::Vertex,
    );

    test_shader!(
        // mov oc, v0
        [
            160, 1, 0, 0, 0, 161, 1, 0, 0, 0, 0, 0, 0, 15, 3, 0, 0, 0, 228, 4, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0
        ],
        &[None, None, None, None, None, None, None, None],
        ShaderType::Fragment,
    );
}
