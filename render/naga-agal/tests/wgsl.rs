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
        .unwrap_or_else(|e| panic!("Validation failed: {e}"));

    let mut writer =
        naga::back::wgsl::Writer::new(&mut out, naga::back::wgsl::WriterFlags::EXPLICIT_TYPES);

    writer.write(module, &module_info).expect("Writing failed");
    out
}

// Making this a macro gives us a better span in 'inta'
macro_rules! test_shader {
    ($shader:expr, $attrs:expr, $shader_type:expr $(,)?) => {
        let module = agal_to_naga(&$shader, $attrs, &[Default::default(); 8]).unwrap();
        let output = to_wgsl(&module);
        insta::assert_snapshot!(output);
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

#[test]
fn test_complex_raytrace() {
    const RAYTRACE_VERTEX: &[u8] = include!("raytrace_vertex.agal");
    const RAYTRACE_FRAGMENT: &[u8] = include!("raytrace_fragment.agal");

    test_shader!(
        RAYTRACE_VERTEX,
        &[
            Some(VertexAttributeFormat::Float4),
            None,
            None,
            None,
            None,
            None,
            None,
            None
        ],
        ShaderType::Vertex
    );

    test_shader!(
        RAYTRACE_FRAGMENT,
        &[None, None, None, None, None, None, None, None],
        ShaderType::Fragment
    );
}

#[test]
fn test_complex_fractal() {
    const FRACTAL_VERTEX: &[u8] = include!("fractal_vertex.agal");
    const FRACTAL_FRAGMENT: &[u8] = include!("fractal_fragment.agal");

    test_shader!(
        FRACTAL_VERTEX,
        &[
            Some(VertexAttributeFormat::Float2),
            Some(VertexAttributeFormat::Float2),
            None,
            None,
            None,
            None,
            None,
            None
        ],
        ShaderType::Vertex
    );

    test_shader!(
        FRACTAL_FRAGMENT,
        &[None, None, None, None, None, None, None, None],
        ShaderType::Fragment
    );
}

#[test]
fn test_relative_load() {
    // mov vt0, vc[va0.x+5]
    // mov vt1, vc[va1.y+6]
    // add op, vt0, vt1
    const RELATIVE_VERTEX: &[u8] = include!("relative_vertex.agal");

    test_shader!(
        RELATIVE_VERTEX,
        &[
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            None,
            None,
            None,
            None,
            None,
            None
        ],
        ShaderType::Vertex
    );
}

#[test]
fn test_misc_opcodes() {
    // log vt0, va0
    // exp vt1, vt0
    // pow vt2, vt1, va0
    // sge vt3, vt2, va0
    // m33 vt4, vc0, vt3
    // m34 vt5, vc2, vt3
    // min vt6, vt5, vt4
    // rsq op, vt6
    const MISC_OPCODES_VERTEX: &[u8] = include!("misc_opcodes_vertex.agal");

    // ddx ft0, v0
    // ddy ft1, ft0
    // kil ft1.x
    // mov oc, ft0
    const MISC_OPCODES_FRAGMENT: &[u8] = include!("misc_opcodes_fragment.agal");

    test_shader!(
        MISC_OPCODES_VERTEX,
        &[
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
            Some(VertexAttributeFormat::Float4),
        ],
        ShaderType::Vertex
    );

    test_shader!(
        MISC_OPCODES_FRAGMENT,
        &[None, None, None, None, None, None, None, None],
        ShaderType::Fragment
    );
}
