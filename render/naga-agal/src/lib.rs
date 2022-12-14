use naga::Module;

mod builder;
mod types;

use builder::NagaBuilder;

const ENTRY_POINT: &str = "main";

pub const MAX_VERTEX_ATTRIBUTES: usize = 8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VertexAttributeFormat {
    Float1,
    Float2,
    Float3,
    Float4,
    Bytes4,
}

#[derive(Debug)]
pub enum Error {
    InvalidHeader,
    InvalidShaderType(u8),
    MissingVertexAttributeData(usize),
    Unimplemented(String),
    ReadError(std::io::Error),
    InvalidOpcode(u32),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ReadError(err)
    }
}

#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

/**
 * Compiles an Adobe AGAL shader to a Naga Module.
 *
 * The `vertex_attributes` parameter is only used when compiling
 * a vertex shader.
 *
 * The returning Module can be passed directly to `wgpu`,
 * or compiled to a particular shader language using a `naga` backend.
 *
 * The shader entrypoint is always named `main`.
 *
 * We compile an AGAL shader as follows:
 *
 * # Vertex Shader
 *
 * * Vertex attributes - AGAL supports up to 8 vertex attributes,
 *   stored in `va0` to `va7`. You must provide the format of each attribute
 *   in the corresponding entry in the `vertex_attributes` array.
 *   Each attribute is mapped to the corresponding binding in the Naga shader
 *   - for example, va3 will have binding id 3.
 *  
 *
 * * Vertex output - An AGAL vertex shader has one main output (a vec4 position),
 *   and 8 varying outputs. The main output is mapped to the Naga 'Position' output,
 *   while each *used* varying register is mapped to a corresponding field in
 *   the Naga output struct. For example, if a vertex shader uses varying registers
 *   2 and 5, then the Naga output struct type will have two members, with binding ids 2 and 5.
 *   If a shader does not write to a varying register, then it is not included in the
 *   Naga output struct type.
 *
 * * Program constants - An AGAL vertex shader has access to 128 program constants.
 *   These are mapped to a single Naga uniform buffer, with a binding id of 0.
 *   Each program constant is a vec4, and are stored in increasing order of register number.
 *
 * # Fragment Shader
 *
 * * Fragment input - An AGAL fragment shader can read from the 8 varying registers
 *   set by the fragment shader. Each *used* varying register is mapped to a corresponding
 *   binding in the Naga input type. For example, if a fragment shader uses varying registers
 *   2 and 5, then the Naga input type will have two members, with binding ids 2 and 5.
 *
 * * Program constants - An AGAL fragment shader has access to 28 program constants.
 *   These are mapped to a single Naga uniform buffer, with a binding id of 1.
 *
 */
pub fn agal_to_naga(
    agal: &[u8],
    vertex_attributes: &[Option<VertexAttributeFormat>; MAX_VERTEX_ATTRIBUTES],
) -> Result<Module, Error> {
    NagaBuilder::process_agal(agal, vertex_attributes)
}
