use std::io::Read;
use std::num::NonZeroU32;

use naga::{
    AddressSpace, ArraySize, Block, BuiltIn, Constant, DerivativeControl, EntryPoint,
    FunctionArgument, FunctionResult, GlobalVariable, ImageClass, ImageDimension, Literal,
    ResourceBinding, Scalar, ShaderStage, StructMember, SwizzleComponent, UnaryOperator,
};
use naga::{BinaryOperator, MathFunction};
use naga::{
    Binding, DerivativeAxis, Expression, Function, Handle, LocalVariable, Module, ScalarKind, Span,
    Statement, Type, TypeInner, VectorSize,
};
use num_traits::FromPrimitive;

use crate::varying::VaryingRegisters;
use crate::{
    types::*, Error, ShaderType, VertexAttributeFormat, MAX_TEXTURES, MAX_VERTEX_ATTRIBUTES,
    SHADER_ENTRY_POINT,
};

const VERTEX_PROGRAM_CONSTANTS: u64 = 128;
const FRAGMENT_PROGRAM_CONSTANTS: u64 = 28;

pub const TEXTURE_SAMPLER_START_BIND_INDEX: u32 = 2;
pub const TEXTURE_START_BIND_INDEX: u32 = 10;

pub type Result<T> = std::result::Result<T, Error>;

const SWIZZLE_XYZW: u8 = 0b11100100;

const SWIZZLE_XXXX: u8 = 0b00000000;
const SWIZZLE_YYYY: u8 = 0b01010101;
const SWIZZLE_ZZZZ: u8 = 0b10101010;
const SWIZZLE_WWWW: u8 = 0b11111111;

#[derive(Copy, Clone)]
struct TextureBindingData {
    // The `Expression::GlobalVariable` corresponding to the texture that we loaded.
    texture_global_var: Handle<Expression>,
    // The `Expression::GlobalVariable` corresponding to bound sampler for the texture
    sampler_global_var: Handle<Expression>,
}

pub(crate) struct NagaBuilder<'a> {
    pub(crate) module: Module,
    pub(crate) func: Function,

    // This evaluate to a Pointer to the temporary 'main' destination location
    // (the output position for a vertex shader, or the output color for a fragment shader)
    // which can be used with Expression::Load and Expression::Store
    // This is needed because an AGAL shader can write to the output register
    // multiple times.
    pub(crate) dest: Handle<Expression>,

    pub(crate) shader_config: ShaderConfig<'a>,

    // Whenever we read from a vertex attribute in a vertex shader
    // for the first time,we fill in the corresponding index
    // of this `Vec` with an `Expression::FunctionArgument`.
    // See `get_vertex_input`
    vertex_input_expressions: Vec<Option<Handle<Expression>>>,

    pub(crate) varying_registers: VaryingRegisters,

    // Whenever we encounter a texture load at a particular index
    // for the first time, we store the expression we generate,
    // as well as the sampler parameters used.
    texture_bindings: [Option<TextureBindingData>; MAX_TEXTURES],

    // Whenever we read from a particular temporary register
    // for the first time, we create a new local variable
    // and store an expression here.
    temporary_registers: Vec<Option<Handle<Expression>>>,

    // An `Expression::GlobalVariables` for the uniform buffer
    // that stores all of the program constants.
    constant_registers: Handle<Expression>,

    // The function return type being built up. Each time a vertex
    // shader writes to a varying register, we add a new member to this
    pub(crate) return_type: Type,

    // The Naga representation of 'vec4f'
    pub(crate) vec4f: Handle<Type>,
    // The Naga representation of 'mat3x3f'
    matrix3x3f: Handle<Type>,
    // The Naga representation of 'mat4x3f'
    matrix4x3f: Handle<Type>,
    // The Naga representation of 'mat4x4f'
    matrix4x4f: Handle<Type>,
    // The Naga representation of `texture_2d<f32>`
    image2d: Handle<Type>,
    // The Naga representation of `texture_cube<f32>`
    imagecube: Handle<Type>,

    // The Naga representation of `f32`
    f32_type: Handle<Type>,

    // The Naga representation of `u32`
    u32_type: Handle<Type>,

    // A stack of if/else blocks, using to push statements
    // into the correct block.
    blocks: Vec<BlockStackEntry>,
}

/// Handles 'if' and 'else' blocks in AGAL bytecode.
/// When we encounter an 'if' opcode, we push an `IfElse` entry onto the block stack.
/// Any subsequent opcodes will be added to the `after_if` block.
/// When we encounter an 'else' opcode, we switch to adding opcodes to the `after_else` block
/// by setting `in_after_if` to false.
/// When we encounter an `eif` opcode, we pop our `IfElse` entry from the stack, and emit
/// a `Statement::If` with the `after_if` and `after_else` blocks.
#[derive(Debug)]
enum BlockStackEntry {
    Normal(Block),
    IfElse {
        after_if: Block,
        after_else: Block,
        in_after_if: bool,
        condition: Handle<Expression>,
    },
}

impl VertexAttributeFormat {
    fn to_naga_type(self, module: &mut Module) -> Handle<Type> {
        if let VertexAttributeFormat::Float1 = self {
            return module.types.insert(
                Type {
                    name: None,
                    inner: TypeInner::Scalar(Scalar::F32),
                },
                Span::UNDEFINED,
            );
        }
        let (size, scalar) = match self {
            VertexAttributeFormat::Float1 => unreachable!(),
            VertexAttributeFormat::Float2 => (VectorSize::Bi, Scalar::F32),
            VertexAttributeFormat::Float3 => (VectorSize::Tri, Scalar::F32),
            VertexAttributeFormat::Float4 => (VectorSize::Quad, Scalar::F32),
            // The conversion is done by wgpu, since we specify
            // `wgpu::VertexFormat::Unorm8x4` in `CurrentPipeline::rebuild_pipeline`
            VertexAttributeFormat::Bytes4 => (VectorSize::Quad, Scalar::F32),
        };

        module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector { size, scalar },
            },
            Span::UNDEFINED,
        )
    }

    fn extend_to_float4(
        &self,
        base_expr: Handle<Expression>,
        builder: &mut NagaBuilder,
    ) -> Result<Handle<Expression>> {
        Ok(match self {
            // This does 'vec4f(my_vec1, 0.0, 0.0, 1.0)', 'vec4f(my_vec2, 0.0, 1.0)',
            // or 'vec4f(my_vec3, 1.0)'
            VertexAttributeFormat::Float1
            | VertexAttributeFormat::Float2
            | VertexAttributeFormat::Float3 => {
                let num_components = match self {
                    VertexAttributeFormat::Float1 => 1,
                    VertexAttributeFormat::Float2 => 2,
                    VertexAttributeFormat::Float3 => 3,
                    _ => unreachable!(),
                };

                let mut components = vec![];
                if num_components == 1 {
                    components.push(base_expr);
                } else {
                    for i in 0..num_components {
                        components.push(builder.evaluate_expr(Expression::AccessIndex {
                            base: base_expr,
                            index: i,
                        }));
                    }
                }

                let const_expr_f32_zero = builder
                    .module
                    .global_expressions
                    .append(Expression::Literal(Literal::F32(0.0)), Span::UNDEFINED);

                let constant_zero = builder.module.constants.append(
                    Constant {
                        name: None,
                        ty: builder.f32_type,
                        init: const_expr_f32_zero,
                    },
                    Span::UNDEFINED,
                );

                for _ in num_components..3 {
                    components.push(
                        builder
                            .func
                            .expressions
                            .append(Expression::Constant(constant_zero), Span::UNDEFINED),
                    );
                }

                let const_expr_f32_1 = builder
                    .module
                    .global_expressions
                    .append(Expression::Literal(Literal::F32(1.0)), Span::UNDEFINED);

                let constant_one = builder.module.constants.append(
                    Constant {
                        name: None,
                        ty: builder.f32_type,
                        init: const_expr_f32_1,
                    },
                    Span::UNDEFINED,
                );
                components.push(
                    builder
                        .func
                        .expressions
                        .append(Expression::Constant(constant_one), Span::UNDEFINED),
                );
                builder.evaluate_expr(Expression::Compose {
                    ty: builder.vec4f,
                    components,
                })
            }
            VertexAttributeFormat::Float4 => base_expr,
            // The conversion is done by wgpu, since we specify
            // `wgpu::VertexFormat::Unorm8x4` in `CurrentPipeline::rebuild_pipeline`
            VertexAttributeFormat::Bytes4 => base_expr,
        })
    }
}

/// Combines information extracted from the AGAL bytecode itself
/// with information provided from the AVM side of ruffle
/// (based on the Context3D methods that ActionScript called)
#[derive(Debug)]
pub struct ShaderConfig<'a> {
    pub shader_type: ShaderType,
    pub vertex_attributes: &'a [Option<VertexAttributeFormat>; 8],
    #[allow(dead_code)] // set but never read
    pub sampler_configs: &'a [SamplerConfig; 8],
    pub version: AgalVersion,
}

#[derive(Debug)]
pub enum AgalVersion {
    Agal1,
    Agal2,
}

struct ParsedBytecode {
    version: AgalVersion,
    shader_type: ShaderType,
    operations: Vec<(Opcode, DestField, SourceField, Source2)>,
}

impl<'a> NagaBuilder<'a> {
    fn parse_bytecode(mut agal: &[u8]) -> Result<ParsedBytecode> {
        let data = &mut agal;

        let mut header = [0; 7];
        data.read_exact(&mut header)?;

        if header[0] != 0xa0 {
            return Err(Error::InvalidHeader);
        }
        let version = u32::from_le_bytes([header[1], header[2], header[3], header[4]]);

        let version = match version {
            1 => AgalVersion::Agal1,
            2 => AgalVersion::Agal2,
            _ => return Err(Error::InvalidVersion(version)),
        };

        if header[5] != 0xa1 {
            return Err(Error::InvalidHeader);
        }

        let shader_type = match header[6] {
            0x00 => ShaderType::Vertex,
            0x01 => ShaderType::Fragment,
            _ => return Err(Error::InvalidShaderType(header[6])),
        };

        let mut operations = Vec::new();

        while !data.is_empty() {
            let mut token = [0; 24];
            data.read_exact(&mut token)?;
            let raw_opcode = u32::from_le_bytes(token[0..4].try_into().unwrap());

            let opcode = Opcode::from_u32(raw_opcode).ok_or(Error::InvalidOpcode(raw_opcode))?;

            let dest = DestField::parse(u32::from_le_bytes(token[4..8].try_into().unwrap()))?;
            let source1 = SourceField::parse(u64::from_le_bytes(token[8..16].try_into().unwrap()))?;

            let source2 = if let Opcode::Tex = opcode {
                Source2::Sampler(SamplerField::parse(u64::from_le_bytes(
                    token[16..24].try_into().unwrap(),
                ))?)
            } else {
                Source2::SourceField(SourceField::parse(u64::from_le_bytes(
                    token[16..24].try_into().unwrap(),
                ))?)
            };
            operations.push((opcode, dest, source1, source2))
        }
        Ok(ParsedBytecode {
            version,
            shader_type,
            operations,
        })
    }

    pub fn extract_sampler_configs(agal: &[u8]) -> Result<[Option<SamplerConfig>; MAX_TEXTURES]> {
        let parsed = Self::parse_bytecode(agal)?;
        let mut sampler_configs = [None; MAX_TEXTURES];
        for (_opcode, _dest, _source1, source2) in parsed.operations {
            if let Source2::Sampler(sampler_field) = source2 {
                // When the 'ignore_sampler' field is set, we do not update the sampler config.
                // The existing sampler value will end up getting used
                // (which comes from a previous Context3D.setSamplerStateAt
                // or Context3D.setProgram call)
                if sampler_field.special.ignore_sampler {
                    continue;
                }

                let sampler_config = SamplerConfig {
                    wrapping: sampler_field.wrapping,
                    filter: sampler_field.filter,
                    mipmap: sampler_field.mipmap,
                };
                let index = sampler_field.reg_num as usize;
                if sampler_configs[index].is_none() {
                    sampler_configs[index] = Some(sampler_config);
                } else if sampler_configs[index] != Some(sampler_config) {
                    return Err(Error::SamplerConfigMismatch {
                        texture: index,
                        old: sampler_configs[index].unwrap(),
                        new: sampler_config,
                    });
                }
            }
        }

        Ok(sampler_configs)
    }

    pub fn build_module(
        agal: &[u8],
        vertex_attributes: &[Option<VertexAttributeFormat>; MAX_VERTEX_ATTRIBUTES],
        sampler_configs: &[SamplerConfig; 8],
    ) -> Result<Module> {
        let parsed = Self::parse_bytecode(agal)?;
        let mut builder = NagaBuilder::new(ShaderConfig {
            shader_type: parsed.shader_type,
            vertex_attributes,
            sampler_configs,
            version: parsed.version,
        });

        for (opcode, dest, source1, source2) in parsed.operations {
            builder.process_opcode(&opcode, &dest, &source1, &source2)?;
        }
        builder.finish()
    }

    // Evaluates a binary operation. The AGAL assembly should always emit a swizzle that only uses
    // a single component, so we can use any component of the source expressions.
    fn boolean_binary_op(
        &mut self,
        left: &SourceField,
        right: &SourceField,
        op: BinaryOperator,
    ) -> Result<Handle<Expression>> {
        let left = self.emit_source_field_load(left, true)?;
        let right = self.emit_source_field_load(right, true)?;

        let res = self.evaluate_expr(Expression::Binary { op, left, right });

        // Cast the boolean result to float 0.0 and 1.0.
        let as_float = self.evaluate_expr(Expression::As {
            expr: res,
            kind: ScalarKind::Float,
            convert: Some(4),
        });

        Ok(as_float)
    }

    fn new(shader_config: ShaderConfig<'a>) -> Self {
        let mut module = Module::default();
        let mut func = Function::default();

        let vec4f = VertexAttributeFormat::Float4.to_naga_type(&mut module);

        let matrix3x3f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: VectorSize::Tri,
                    rows: VectorSize::Tri,
                    scalar: Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let matrix4x3f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: VectorSize::Tri,
                    rows: VectorSize::Quad,
                    scalar: Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let matrix4x4f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: VectorSize::Quad,
                    rows: VectorSize::Quad,
                    scalar: Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let image2d = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Image {
                    dim: ImageDimension::D2,
                    arrayed: false,
                    class: ImageClass::Sampled {
                        kind: ScalarKind::Float,
                        multi: false,
                    },
                },
            },
            Span::UNDEFINED,
        );

        let imagecube = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Image {
                    dim: ImageDimension::Cube,
                    arrayed: false,
                    class: ImageClass::Sampled {
                        kind: ScalarKind::Float,
                        multi: false,
                    },
                },
            },
            Span::UNDEFINED,
        );

        let f32_type = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Scalar(Scalar::F32),
            },
            Span::UNDEFINED,
        );

        let u32_type = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Scalar(Scalar::U32),
            },
            Span::UNDEFINED,
        );

        // The return type always has at least one component - the vec4f that's the 'main'
        // output of our shader (the position for the vertex shader, and the color for the fragment shader)
        let return_type = match shader_config.shader_type {
            ShaderType::Vertex => Type {
                name: None,
                inner: TypeInner::Struct {
                    members: vec![StructMember {
                        name: None,
                        ty: vec4f,
                        binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                        offset: 0,
                    }],
                    span: 16,
                },
            },
            ShaderType::Fragment => Type {
                name: None,
                inner: TypeInner::Struct {
                    members: vec![StructMember {
                        name: None,
                        ty: vec4f,
                        binding: Some(Binding::Location {
                            location: 0,
                            interpolation: None,
                            sampling: None,
                            second_blend_source: false,
                        }),
                        offset: 0,
                    }],
                    span: 16,
                },
            },
        };

        match shader_config.shader_type {
            ShaderType::Vertex => {
                func.result = Some(FunctionResult {
                    ty: vec4f,
                    binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
                });
            }
            ShaderType::Fragment => {
                func.result = Some(FunctionResult {
                    ty: vec4f,
                    binding: Some(Binding::Location {
                        location: 0,
                        interpolation: None,
                        sampling: None,
                        second_blend_source: false,
                    }),
                });
            }
        }

        // Holds the value we're going to return.
        // This corresponds to RegisterType::Output
        let output_temp_handle = func.local_variables.append(
            LocalVariable {
                name: Some("dest_temp".to_string()),
                ty: vec4f,
                init: None,
            },
            Span::UNDEFINED,
        );
        let dest = func.expressions.append(
            Expression::LocalVariable(output_temp_handle),
            Span::UNDEFINED,
        );

        let binding_num = match shader_config.shader_type {
            ShaderType::Vertex => 0,
            ShaderType::Fragment => 1,
        };

        let constant_registers_global = module.global_variables.append(
            GlobalVariable {
                name: Some("constant_registers".to_string()),
                space: naga::AddressSpace::Uniform,
                binding: Some(naga::ResourceBinding {
                    group: 0,
                    binding: binding_num,
                }),
                ty: module.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Array {
                            base: vec4f,
                            size: ArraySize::Constant(
                                NonZeroU32::new(match shader_config.shader_type {
                                    ShaderType::Vertex => VERTEX_PROGRAM_CONSTANTS as u32,
                                    ShaderType::Fragment => FRAGMENT_PROGRAM_CONSTANTS as u32,
                                })
                                .unwrap(),
                            ),
                            stride: std::mem::size_of::<f32>() as u32 * 4,
                        },
                    },
                    Span::UNDEFINED,
                ),
                init: None,
            },
            Span::UNDEFINED,
        );

        let constant_registers = func.expressions.append(
            Expression::GlobalVariable(constant_registers_global),
            Span::UNDEFINED,
        );

        // FIXME - expose this to the wgpu code
        let num_temporaries = match shader_config.version {
            AgalVersion::Agal1 => 8,
            AgalVersion::Agal2 => 26,
        };

        NagaBuilder {
            module,
            func,
            dest,
            shader_config,
            vertex_input_expressions: vec![],
            varying_registers: Default::default(),
            return_type,
            matrix3x3f,
            matrix4x3f,
            matrix4x4f,
            vec4f,
            f32_type,
            u32_type,
            constant_registers,
            texture_bindings: [None; 8],
            temporary_registers: vec![None; num_temporaries],
            image2d,
            imagecube,
            blocks: vec![BlockStackEntry::Normal(Block::new())],
        }
    }

    fn get_vertex_input(&mut self, index: usize) -> Result<Handle<Expression>> {
        if index >= self.vertex_input_expressions.len() {
            self.vertex_input_expressions.resize(index + 1, None);
        }

        if self.vertex_input_expressions[index].is_none() {
            let ty = self.shader_config.vertex_attributes[index]
                .as_ref()
                .ok_or(Error::MissingVertexAttributeData(index))?
                .to_naga_type(&mut self.module);

            // Function arguments might not be in the same order as the
            // corresponding binding indices (e.g. the first argument might have binding '2').
            // However, we only access the `FunctionArgument` expression through the `vertex_input_expressions`
            // vec, which is indexed by the binding index.
            self.func.arguments.push(FunctionArgument {
                name: None,
                ty,
                binding: Some(Binding::Location {
                    location: index as u32,
                    interpolation: None,
                    sampling: None,
                    second_blend_source: false,
                }),
            });

            let arg_index = self.func.arguments.len() - 1;

            // Arguments map one-tom-one to vertex attributes.
            let expr = self.func.expressions.append(
                Expression::FunctionArgument(arg_index as u32),
                Span::UNDEFINED,
            );
            self.vertex_input_expressions[index] = Some(expr);
        }
        Ok(self.vertex_input_expressions[index].unwrap())
    }

    fn get_temporary_register(&mut self, index: usize) -> Result<Handle<Expression>> {
        if self.temporary_registers[index].is_none() {
            let local = self.func.local_variables.append(
                LocalVariable {
                    name: Some(format!("temporary{}", index)),
                    ty: self.vec4f,
                    init: None,
                },
                Span::UNDEFINED,
            );

            let expr = self
                .func
                .expressions
                .append(Expression::LocalVariable(local), Span::UNDEFINED);
            self.temporary_registers[index] = Some(expr);
        }
        Ok(self.temporary_registers[index].unwrap())
    }

    fn emit_const_register_load(&mut self, index: usize) -> Result<Handle<Expression>> {
        let const_value_expr = self.module.global_expressions.append(
            Expression::Literal(Literal::U32(index as u32)),
            Span::UNDEFINED,
        );
        let index_const = self.module.constants.append(
            Constant {
                name: None,
                ty: self.u32_type,
                init: const_value_expr,
            },
            Span::UNDEFINED,
        );
        let index_expr = self
            .func
            .expressions
            .append(Expression::Constant(index_const), Span::UNDEFINED);

        let register_pointer = self.func.expressions.append(
            Expression::Access {
                base: self.constant_registers,
                index: index_expr,
            },
            Span::UNDEFINED,
        );

        Ok(self.evaluate_expr(Expression::Load {
            pointer: register_pointer,
        }))
    }

    pub(crate) fn emit_varying_load(&mut self, index: usize) -> Result<Handle<Expression>> {
        // A LocalVariable evaluates to a pointer, so we need to load it
        let varying_expr = self.get_varying_pointer(index)?;
        Ok(match self.shader_config.shader_type {
            ShaderType::Vertex => self.evaluate_expr(Expression::Load {
                pointer: varying_expr,
            }),
            ShaderType::Fragment => varying_expr,
        })
    }

    fn emit_texture_load(
        &mut self,
        index: usize,
        dimension: Dimension,
    ) -> Result<TextureBindingData> {
        if self.texture_bindings[index].is_none() {
            let global_var = self.module.global_variables.append(
                GlobalVariable {
                    name: Some(format!("texture{}", index)),
                    space: AddressSpace::Handle,
                    binding: Some(ResourceBinding {
                        group: 0,
                        binding: TEXTURE_START_BIND_INDEX + index as u32,
                    }),
                    // Note - we assume that a given texture is always sampled with the same dimension
                    // (2d or cube)
                    ty: match dimension {
                        Dimension::TwoD => self.image2d,
                        Dimension::Cube => self.imagecube,
                    },
                    init: None,
                },
                Span::UNDEFINED,
            );

            let sampler_var = self.module.global_variables.append(
                GlobalVariable {
                    name: Some(format!("sampler{}", index)),
                    space: naga::AddressSpace::Handle,
                    binding: Some(naga::ResourceBinding {
                        group: 0,
                        binding: TEXTURE_SAMPLER_START_BIND_INDEX + index as u32,
                    }),
                    ty: self.module.types.insert(
                        Type {
                            name: None,
                            inner: TypeInner::Sampler { comparison: false },
                        },
                        Span::UNDEFINED,
                    ),
                    init: None,
                },
                Span::UNDEFINED,
            );

            self.texture_bindings[index] = Some(TextureBindingData {
                texture_global_var: self
                    .func
                    .expressions
                    .append(Expression::GlobalVariable(global_var), Span::UNDEFINED),
                sampler_global_var: self
                    .func
                    .expressions
                    .append(Expression::GlobalVariable(sampler_var), Span::UNDEFINED),
            });
        }
        let data = self.texture_bindings[index].as_ref().unwrap();
        Ok(*data)
    }

    fn emit_source_field_load(
        &mut self,
        source: &SourceField,
        extend_to_vec4: bool,
    ) -> Result<Handle<Expression>> {
        self.emit_source_field_load_with_swizzle_out(source, extend_to_vec4, VectorSize::Quad)
    }

    fn emit_source_field_load_with_swizzle_out(
        &mut self,
        source: &SourceField,
        extend_to_vec4: bool,
        output: VectorSize,
    ) -> Result<Handle<Expression>> {
        let mut load_register = |register_type: &RegisterType, reg_num| {
            match register_type {
                // We can use a function argument directly - we don't need
                // a separate Expression::Load
                RegisterType::Attribute => Ok((
                    self.get_vertex_input(reg_num)?,
                    self.shader_config.vertex_attributes[reg_num]
                        .ok_or(Error::MissingVertexAttributeData(reg_num))?,
                )),
                RegisterType::Varying => Ok((
                    self.emit_varying_load(reg_num)?,
                    VertexAttributeFormat::Float4,
                )),
                RegisterType::Constant => Ok((
                    self.emit_const_register_load(reg_num)?,
                    // Constants are always a vec4<f32>
                    VertexAttributeFormat::Float4,
                )),
                RegisterType::Temporary => Ok({
                    let temp = self.get_temporary_register(reg_num)?;
                    (
                        self.evaluate_expr(Expression::Load { pointer: temp }),
                        VertexAttributeFormat::Float4,
                    )
                }),
                _ => Err(Error::Unimplemented(format!(
                    "Unimplemented source reg type {:?}",
                    source.register_type
                ))),
            }
        };

        let (mut base_expr, source_type) = match source.direct_mode {
            DirectMode::Direct => load_register(&source.register_type, source.reg_num as usize)?,
            DirectMode::Indirect => {
                // Handle an indirect register load, e.g. `vc[va0.x + offset]`
                // Indirect loads allow loading from a dynamically computed register index.
                // This dynamic index is computed as 'regN.X + offset', where 'regN' is a normal
                // register (e.g. 'va0'), and 'X' is a component ('X, 'Y', Z', or 'W').
                // Currently, we only support this when the 'outer' (non-index) register is
                // a constant register, since we always access constant registers through
                // an array access.
                match source.register_type {
                    RegisterType::Constant => {
                        // Load the index register (e.g. 'va0') as normal, and access the component
                        // given by 'index_select' (e.g. 'x'). This is 'va0.x' in the above example.
                        let (base_index, _format) =
                            load_register(&source.index_type, source.reg_num as usize)?;
                        let index_expr = self.evaluate_expr(Expression::AccessIndex {
                            base: base_index,
                            index: source.index_select as u32,
                        });

                        // Convert to an integer, since we're going to be indexing an array
                        let index_integer = self.evaluate_expr(Expression::As {
                            expr: index_expr,
                            kind: ScalarKind::Uint,
                            convert: Some(4),
                        });

                        let const_indirect_offset = self.module.global_expressions.append(
                            Expression::Literal(Literal::U32(source.indirect_offset as u32)),
                            Span::UNDEFINED,
                        );

                        let offset_constant = self.module.constants.append(
                            Constant {
                                name: None,
                                ty: self.u32_type,
                                init: const_indirect_offset,
                            },
                            Span::UNDEFINED,
                        );
                        let offset_constant = self
                            .func
                            .expressions
                            .append(Expression::Constant(offset_constant), Span::UNDEFINED);

                        // Add the offset to the loaded value. THis gives us `va0.x + offset` in the above example.
                        let index_with_offset = self.evaluate_expr(Expression::Binary {
                            op: BinaryOperator::Add,
                            left: index_integer,
                            right: offset_constant,
                        });

                        let register_pointer = self.func.expressions.append(
                            Expression::Access {
                                base: self.constant_registers,
                                index: index_with_offset,
                            },
                            Span::UNDEFINED,
                        );

                        // Perform the actual load, giving us 'vc[va0.x + offset]' in the above example.
                        (
                            self.evaluate_expr(Expression::Load {
                                pointer: register_pointer,
                            }),
                            // Constants are always a vec4<f32>
                            VertexAttributeFormat::Float4,
                        )
                    }
                    _ => {
                        return Err(Error::Unimplemented(format!(
                            "Unimplemented register type in indirect mode {:?}",
                            source.register_type
                        )))
                    }
                }
            }
        };

        if extend_to_vec4 && source_type != VertexAttributeFormat::Float4 {
            base_expr = source_type.extend_to_float4(base_expr, self)?;
        }

        // This is a no-op swizzle - we can just return the base expression
        if source.swizzle == SWIZZLE_XYZW && output == VectorSize::Quad {
            return Ok(base_expr);
        }

        let swizzle_flags = [
            source.swizzle & 0b11,
            (source.swizzle >> 2) & 0b11,
            (source.swizzle >> 4) & 0b11,
            (source.swizzle >> 6) & 0b11,
        ];
        let swizzle_components: [SwizzleComponent; 4] = swizzle_flags
            .into_iter()
            .map(|flag| match flag {
                0b00 => SwizzleComponent::X,
                0b01 => SwizzleComponent::Y,
                0b10 => SwizzleComponent::Z,
                0b11 => SwizzleComponent::W,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Ok(self.evaluate_expr(Expression::Swizzle {
            size: output,
            vector: base_expr,
            pattern: swizzle_components,
        }))
    }

    fn emit_dest_store(&mut self, dest: &DestField, expr: Handle<Expression>) -> Result<()> {
        let base_expr = match dest.register_type {
            RegisterType::Output => self.dest,
            RegisterType::Varying => self.get_varying_pointer(dest.reg_num as usize)?,
            RegisterType::Temporary => self.get_temporary_register(dest.reg_num as usize)?,
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unimplemented dest reg type: {dest:?}",
                )))
            }
        };

        // TODO - ideally, Naga would be able to tell us this information.
        let source_is_scalar = matches!(
            self.func.expressions[expr],
            Expression::Math {
                fun: MathFunction::Dot,
                ..
            }
        );

        // Optimization - use a Store instead of writing individual fields
        // when we're writing to the entire output register.
        if dest.write_mask.is_all() && !source_is_scalar {
            self.push_statement(Statement::Store {
                pointer: base_expr,
                value: expr,
            });
        } else {
            // A scalar write occurs when we have exactly one component in the dest write mask.
            let scalar_write = [Mask::X, Mask::Y, Mask::Z, Mask::W]
                .into_iter()
                .filter(|mask| dest.write_mask.contains(*mask))
                .count()
                == 1;

            for (i, mask) in [(0, Mask::X), (1, Mask::Y), (2, Mask::Z), (3, Mask::W)] {
                if dest.write_mask.contains(mask) {
                    let source_component = if scalar_write || source_is_scalar {
                        if source_is_scalar {
                            expr
                        } else {
                            // Grab the first component of the source expression - all of them should be
                            // the same when doing a scalar write.
                            self.evaluate_expr(Expression::AccessIndex {
                                base: expr,
                                index: 0,
                            })
                        }
                    } else {
                        self.evaluate_expr(Expression::AccessIndex {
                            base: expr,
                            index: i,
                        })
                    };
                    let dest_component = self.evaluate_expr(Expression::AccessIndex {
                        base: base_expr,
                        index: i,
                    });
                    self.push_statement(Statement::Store {
                        pointer: dest_component,
                        value: source_component,
                    });
                }
            }
        }
        Ok(())
    }

    /// Creates a `Statement::Emit` covering `expr`
    pub(crate) fn evaluate_expr(&mut self, expr: Expression) -> Handle<Expression> {
        let prev_len = self.func.expressions.len();
        let expr = self.func.expressions.append(expr, Span::UNDEFINED);
        let range = self.func.expressions.range_from(prev_len);
        self.push_statement(Statement::Emit(range));
        expr
    }

    /// Pushes a statement, taking into account our current 'if' block.
    /// Use this instead of `self.func.body.push`
    fn push_statement(&mut self, stmt: Statement) {
        let block = match self.blocks.last_mut().unwrap() {
            BlockStackEntry::Normal(block) => block,
            BlockStackEntry::IfElse {
                after_if,
                after_else,
                in_after_if,
                condition: _,
            } => {
                if *in_after_if {
                    after_if
                } else {
                    after_else
                }
            }
        };
        block.push(stmt, Span::UNDEFINED);
    }

    fn process_opcode(
        &mut self,
        opcode: &Opcode,
        dest: &DestField,
        source1: &SourceField,
        source2: &Source2,
    ) -> Result<()> {
        match opcode {
            // Copy the source register to the destination register
            Opcode::Mov => {
                let source = self.emit_source_field_load(source1, true)?;
                self.emit_dest_store(dest, source)?;
            }
            Opcode::Mul => {
                let source2 = match source2 {
                    Source2::SourceField(source2) => source2,
                    _ => unreachable!(),
                };
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2, true)?;
                let expr = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Multiply,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, expr)?;
            }
            // Perform 'M * v', where M is a matrix, and 'v' is a column vector.
            Opcode::M33 | Opcode::M34 | Opcode::M44 => {
                let source2 = match source2 {
                    Source2::SourceField(source2) => source2,
                    _ => unreachable!(),
                };

                let (num_rows, ty, vec_size, out_size) = match opcode {
                    Opcode::M33 => (
                        3u8,
                        self.matrix3x3f,
                        VectorSize::Tri,
                        VertexAttributeFormat::Float3,
                    ),
                    Opcode::M34 => (
                        3,
                        self.matrix4x3f,
                        VectorSize::Quad,
                        VertexAttributeFormat::Float3,
                    ),
                    Opcode::M44 => (
                        4,
                        self.matrix4x4f,
                        VectorSize::Quad,
                        VertexAttributeFormat::Float4,
                    ),
                    _ => unreachable!(),
                };

                // Read each row of the matrix
                let mut components = Vec::with_capacity(num_rows.into());
                for i in 0..num_rows {
                    let modified_source_field = match source2.direct_mode {
                        DirectMode::Direct => SourceField {
                            reg_num: source2.reg_num + (i as u16),
                            ..source2.clone()
                        },
                        DirectMode::Indirect => SourceField {
                            indirect_offset: source2.indirect_offset + i,
                            ..source2.clone()
                        },
                    };
                    let source2_row = self.emit_source_field_load_with_swizzle_out(
                        &modified_source_field,
                        false,
                        vec_size,
                    )?;
                    components.push(source2_row);
                }

                // FIXME - The naga spv backend hits an 'unreachable!'
                // if we don't create a Statement::Emit for each of these,
                // even though validation passes. We should investigate this
                // and report it upstream.
                let matrix = self.evaluate_expr(Expression::Compose { ty, components });

                // Naga interprets each component of the matrix as a *column*.
                // However, the matrix is stored in memory as a *row*, so we need
                // to transpose it.
                let matrix = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Transpose,
                    arg: matrix,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });

                let vector =
                    self.emit_source_field_load_with_swizzle_out(source1, true, vec_size)?;

                let multiply = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Multiply,
                    left: matrix,
                    right: vector,
                });

                let extended_out = out_size.extend_to_float4(multiply, self)?;

                self.emit_dest_store(dest, extended_out)?;
            }
            Opcode::Tex => {
                let sampler_field = source2.assert_sampler();

                let texture_id = sampler_field.reg_num;
                if sampler_field.reg_type != RegisterType::Sampler {
                    panic!("Invalid sample register type {:?}", sampler_field);
                }

                let coord = self.emit_source_field_load(source1, false)?;
                let coord = match sampler_field.dimension {
                    Dimension::TwoD => {
                        self.evaluate_expr(Expression::Swizzle {
                            size: VectorSize::Bi,
                            vector: coord,
                            // Only the first two components matter here
                            pattern: [
                                SwizzleComponent::X,
                                SwizzleComponent::Y,
                                SwizzleComponent::W,
                                SwizzleComponent::W,
                            ],
                        })
                    }
                    Dimension::Cube => {
                        self.evaluate_expr(Expression::Swizzle {
                            size: VectorSize::Tri,
                            vector: coord,
                            // Only the first three components matter here
                            pattern: [
                                SwizzleComponent::X,
                                SwizzleComponent::Y,
                                SwizzleComponent::Z,
                                SwizzleComponent::W,
                            ],
                        })
                    }
                };

                let texture_binding =
                    self.emit_texture_load(texture_id as usize, sampler_field.dimension)?;
                let tex = self.evaluate_expr(Expression::ImageSample {
                    image: texture_binding.texture_global_var,
                    sampler: texture_binding.sampler_global_var,
                    coordinate: coord,
                    array_index: None,
                    offset: None,
                    // FIXME - get this from 'LOD_bias' in the sampler field
                    level: naga::SampleLevel::Auto,
                    depth_ref: None,
                    gather: None,
                });
                self.emit_dest_store(dest, tex)?;
            }
            Opcode::Cos => {
                let source = self.emit_source_field_load(source1, true)?;
                let cos = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Cos,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, cos)?;
            }
            Opcode::Sin => {
                let source = self.emit_source_field_load(source1, true)?;
                let sin = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Sin,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, sin)?;
            }
            Opcode::Add => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let add = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Add,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, add)?;
            }
            Opcode::Sub => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let sub = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Subtract,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, sub)?;
            }
            Opcode::Div => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let div = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Divide,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, div)?;
            }
            Opcode::Min => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let max = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Min,
                    arg: source1,
                    arg1: Some(source2),
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, max)?;
            }
            Opcode::Max => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let max = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Max,
                    arg: source1,
                    arg1: Some(source2),
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, max)?;
            }
            Opcode::Nrm => {
                // This opcode only looks at the first three components of the source, so load it as a Vec3
                let source =
                    self.emit_source_field_load_with_swizzle_out(source1, true, VectorSize::Tri)?;
                let nrm = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Normalize,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, nrm)?;
            }
            Opcode::Rcp => {
                let source = self.emit_source_field_load(source1, true)?;

                let f32_one = self
                    .func
                    .expressions
                    .append(Expression::Literal(Literal::F32(1.0)), Span::UNDEFINED);

                let vec_one = self.evaluate_expr(Expression::Splat {
                    size: naga::VectorSize::Quad,
                    value: f32_one,
                });

                // Perform 'vec4(1.0, 1.0, 1.0. 1.0) / src'
                let rcp = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Divide,
                    left: vec_one,
                    right: source,
                });
                self.emit_dest_store(dest, rcp)?;
            }
            Opcode::Sqt => {
                let source = self.emit_source_field_load(source1, true)?;
                let sqt = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Sqrt,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, sqt)?;
            }
            Opcode::Rsq => {
                let source = self.emit_source_field_load(source1, true)?;
                let sqt = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::InverseSqrt,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, sqt)?;
            }
            Opcode::Crs => {
                // Zero-extend if necessary, so that we have two three-component input vectors for a cross product.
                let source1 =
                    self.emit_source_field_load_with_swizzle_out(source1, true, VectorSize::Tri)?;
                let source2 = self.emit_source_field_load_with_swizzle_out(
                    source2.assert_source_field(),
                    true,
                    VectorSize::Tri,
                )?;
                let crs = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Cross,
                    arg: source1,
                    arg1: Some(source2),
                    arg2: None,
                    arg3: None,
                });
                let extended = VertexAttributeFormat::Float3.extend_to_float4(crs, self)?;
                self.emit_dest_store(dest, extended)?;
            }
            Opcode::Ife | Opcode::Ine | Opcode::Ifg | Opcode::Ifl => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let condition = self.evaluate_expr(Expression::Binary {
                    op: match opcode {
                        Opcode::Ife => BinaryOperator::Equal,
                        Opcode::Ine => BinaryOperator::NotEqual,
                        Opcode::Ifg => BinaryOperator::Greater,
                        Opcode::Ifl => BinaryOperator::Less,
                        _ => unreachable!(),
                    },
                    left: source1,
                    right: source2,
                });

                let all_match = self.evaluate_expr(Expression::Relational {
                    fun: naga::RelationalFunction::All,
                    argument: condition,
                });

                self.blocks.push(BlockStackEntry::IfElse {
                    after_if: Block::new(),
                    after_else: Block::new(),
                    in_after_if: true,
                    condition: all_match,
                })
            }
            Opcode::Els => {
                if let BlockStackEntry::IfElse {
                    after_if: _,
                    after_else: _,
                    in_after_if,
                    condition: _,
                } = self.blocks.last_mut().unwrap()
                {
                    if !*in_after_if {
                        panic!("Multiple' els' opcodes for single 'if' opcode");
                    }
                    *in_after_if = false;
                } else {
                    unreachable!()
                }
            }
            Opcode::Eif => {
                let block = self.blocks.pop().unwrap();

                match block {
                    BlockStackEntry::IfElse {
                        after_if,
                        after_else,
                        in_after_if: _,
                        condition,
                    } => {
                        self.push_statement(Statement::If {
                            condition,
                            // The opcodes occurig directly after the 'if' opcode
                            // get run if the condition is true
                            accept: after_if,
                            // The opcodes occurring directly after the 'els' opcode
                            // get run if the condition is false
                            reject: after_else,
                        });
                    }
                    BlockStackEntry::Normal(block) => {
                        panic!("Eif opcode without matching 'if': {:?}", block)
                    }
                }
            }
            Opcode::Dp3 => {
                let source2 = source2.assert_source_field();

                let source1 =
                    self.emit_source_field_load_with_swizzle_out(source1, false, VectorSize::Tri)?;
                let source2 =
                    self.emit_source_field_load_with_swizzle_out(source2, false, VectorSize::Tri)?;

                let dp3 = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Dot,
                    arg: source1,
                    arg1: Some(source2),
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, dp3)?;
            }
            Opcode::Dp4 => {
                let source2 = source2.assert_source_field();

                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2, true)?;

                let dp3 = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Dot,
                    arg: source1,
                    arg1: Some(source2),
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, dp3)?;
            }
            Opcode::Neg => {
                let source = self.emit_source_field_load(source1, true)?;
                let neg = self.evaluate_expr(Expression::Unary {
                    op: UnaryOperator::Negate,
                    expr: source,
                });
                self.emit_dest_store(dest, neg)?;
            }
            Opcode::Sge => {
                let result = self.boolean_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::GreaterEqual,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Slt => {
                let result = self.boolean_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::Less,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Seq => {
                let result = self.boolean_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::Equal,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Sne => {
                let result = self.boolean_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::NotEqual,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Sat => {
                let source = self.emit_source_field_load(source1, true)?;
                let sat = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Saturate,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, sat)?;
            }
            Opcode::Frc => {
                let source = self.emit_source_field_load(source1, true)?;
                let frc = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Fract,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, frc)?;
            }
            Opcode::Abs => {
                let source = self.emit_source_field_load(source1, true)?;
                let abs = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Abs,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, abs)?;
            }
            Opcode::Pow => {
                let source1 = self.emit_source_field_load(source1, true)?;
                let source2 = self.emit_source_field_load(source2.assert_source_field(), true)?;
                let pow = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Pow,
                    arg: source1,
                    arg1: Some(source2),
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, pow)?;
            }
            Opcode::Log => {
                let source = self.emit_source_field_load(source1, true)?;
                let log = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Log2,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, log)?;
            }
            Opcode::Exp => {
                let source = self.emit_source_field_load(source1, true)?;
                let exp = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Exp2,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, exp)?;
            }
            Opcode::Ddx => {
                let source = self.emit_source_field_load(source1, true)?;
                let derivative = self.evaluate_expr(Expression::Derivative {
                    axis: DerivativeAxis::X,
                    expr: source,
                    ctrl: DerivativeControl::None,
                });
                self.emit_dest_store(dest, derivative)?;
            }
            Opcode::Ddy => {
                let source = self.emit_source_field_load(source1, true)?;
                let derivative = self.evaluate_expr(Expression::Derivative {
                    axis: DerivativeAxis::Y,
                    expr: source,
                    ctrl: DerivativeControl::None,
                });
                self.emit_dest_store(dest, derivative)?;
            }
            Opcode::Kil => {
                if ![SWIZZLE_XXXX, SWIZZLE_YYYY, SWIZZLE_ZZZZ, SWIZZLE_WWWW]
                    .contains(&source1.swizzle)
                {
                    panic!(
                        "Kil op with source swizzle involving multiple distinct components: {:?}",
                        source1.swizzle
                    );
                }

                let source = self.emit_source_field_load(source1, false)?;

                // Grab single scalar component of source.
                let source = self.evaluate_expr(Expression::AccessIndex {
                    base: source,
                    index: 0,
                });

                let constant_f32_zero = self
                    .module
                    .global_expressions
                    .append(Expression::Literal(Literal::F32(0.0)), Span::UNDEFINED);

                // Check `source < 0.0`.
                let constant_zero = self.module.constants.append(
                    Constant {
                        name: None,
                        ty: self.f32_type,
                        init: constant_f32_zero,
                    },
                    Span::UNDEFINED,
                );
                let zero = self
                    .func
                    .expressions
                    .append(Expression::Constant(constant_zero), Span::UNDEFINED);
                let less_than_zero = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Less,
                    left: source,
                    right: zero,
                });

                // If `source < 0.0`, kill fragment.
                self.push_statement(Statement::If {
                    condition: less_than_zero,
                    accept: Block::from_vec(vec![Statement::Kill]),
                    reject: Block::new(),
                });
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Module> {
        // We're consuming 'self', so just store store garbage here so that we can continue
        // to use methods on 'self'
        let return_ty = std::mem::replace(
            &mut self.return_type,
            Type {
                name: None,
                inner: TypeInner::Scalar(Scalar::F32),
            },
        );

        // Finalize the return type, and do emit the actual return
        let return_ty = self.module.types.insert(return_ty, Span::UNDEFINED);
        self.func.result = Some(FunctionResult {
            ty: return_ty,
            binding: None,
        });

        let return_expr = self.build_output_expr(return_ty)?;
        self.push_statement(Statement::Return {
            value: Some(return_expr),
        });

        let block = match self.blocks.pop().unwrap() {
            BlockStackEntry::Normal(block) => block,
            block => panic!("Unfinished if statement: {:?}", block),
        };

        if !self.blocks.is_empty() {
            panic!("Unbalanced blocks: {:?}", self.blocks);
        }
        if !self.func.body.is_empty() {
            panic!("Incorrectly wrote to function body: {:?}", self.func.body);
        }
        self.func.body = block;

        let entry_point = EntryPoint {
            name: SHADER_ENTRY_POINT.to_string(),
            stage: match self.shader_config.shader_type {
                ShaderType::Vertex => ShaderStage::Vertex,
                ShaderType::Fragment => ShaderStage::Fragment,
            },
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: self.func,
        };

        self.module.entry_points.push(entry_point);
        Ok(self.module)
    }
}
