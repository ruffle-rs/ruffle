use std::io::Read;

use naga::{
    AddressSpace, ArraySize, Block, BuiltIn, Constant, ConstantInner, EntryPoint, FunctionArgument,
    FunctionResult, GlobalVariable, ImageClass, ImageDimension, Interpolation, ResourceBinding,
    ScalarValue, ShaderStage, StructMember, SwizzleComponent, UnaryOperator,
};
use naga::{BinaryOperator, MathFunction};
use naga::{
    Binding, Expression, Function, Handle, LocalVariable, Module, ScalarKind, Span, Statement,
    Type, TypeInner, VectorSize,
};
use num_traits::FromPrimitive;

use crate::{
    types::*, Error, ShaderType, VertexAttributeFormat, MAX_VERTEX_ATTRIBUTES, SHADER_ENTRY_POINT,
};

const VERTEX_PROGRAM_CONTANTS: u64 = 128;
const FRAGMENT_PROGRAM_CONSTANTS: u64 = 28;

const SAMPLER_REPEAT_LINEAR: usize = 0;
const SAMPLER_REPEAT_NEAREST: usize = 1;
const SAMPLER_CLAMP_LINEAR: usize = 2;
const SAMPLER_CLAMP_NEAREST: usize = 3;

const TEXTURE_SAMPLER_START_BIND_INDEX: u32 = 2;
const TEXTURE_START_BIND_INDEX: u32 = 6;

pub type Result<T> = std::result::Result<T, Error>;

const SWIZZLE_XYZW: u8 = 0b11100100;

const SWIZZLE_XXXX: u8 = 0b00000000;
const SWIZZLE_YYYY: u8 = 0b01010101;
const SWIZZLE_ZZZZ: u8 = 0b10101010;
const SWIZZLE_WWWW: u8 = 0b11111111;

struct TextureSamplers {
    repeat_linear: Handle<Expression>,
    repeat_nearest: Handle<Expression>,
    clamp_linear: Handle<Expression>,
    clamp_nearest: Handle<Expression>,
}

pub(crate) struct NagaBuilder<'a> {
    module: Module,
    func: Function,

    // This evaluate to a Pointer to the temporary 'main' destiation location
    // (the output position for a vertex shader, or the output color for a fragment shader)
    // which can be used with Expression::Load and Expression::Store
    // This is needed because an AGAL shader can write to the output register
    // multiple times.
    dest: Handle<Expression>,

    shader_config: ShaderConfig<'a>,

    // Whenever we read from a vertex attribute in a vertex shader
    // for the first time,we fill in the corresponding index
    // of this `Vec` with an `Expression::FunctionArgument`.
    // See `get_vertex_input`
    vertex_input_expressions: Vec<Option<Handle<Expression>>>,

    // Whenever we write to a varying register in a vertex shader
    // or read from a varying register in a fragment shader
    // (for the first time), we store the created `Expression` here.
    // See `get_varying_pointer`
    varying_pointers: Vec<Option<Handle<Expression>>>,

    // Whenever we encounter a texture load at a particular index
    // for the first time, we store an `Expression::GlobalVariable`
    // here corresponding to the texture that we loaded.
    texture_bindings: [Option<Handle<Expression>>; 8],

    // Whenever we read from a particular temporary register
    // for the first time, we create a new local variable
    // and store an expression here.
    temporary_registers: Vec<Option<Handle<Expression>>>,

    // An `Expression::GlobalVariables` for the uniform buffer
    // that stores all of the program constants.
    constant_registers: Handle<Expression>,

    // The function return type being built up. Each time a vertex
    // shader writes to a varying register, we add a new member to this
    return_type: Type,

    // The Naga representation of 'vec4f'
    vec4f: Handle<Type>,
    // The Naga representation of 'mat4x4f'
    matrix4x4f: Handle<Type>,
    // The Naga representation of `texture_2d<f32>`
    image2d: Handle<Type>,
    // The Naga representation of `texture_cube<f32>`
    imagecube: Handle<Type>,

    // For a fragment shader, our 4 bound texture samplers
    texture_samplers: Option<TextureSamplers>,

    // A stack of if/else blocks, using to push statements
    // into the correct block.
    blocks: Vec<BlockStackEntry>,
}

/// Handles 'if' and 'else' blocks in AGAL bytecode.
/// When we encounter an 'if' opcode, we push an `IfElse` entry onto the block stack.
/// Any subsequent opcodes will be added to the `after_if` block.
/// When we encounter an 'else' opcode, we switch to adding opcodes to the `after_else` block
/// by setting `in_after_if` to false.
/// When we encouter an `eif` opcode, we pop our `IfElse` entry from the stack, and emit
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
                    inner: TypeInner::Scalar {
                        kind: ScalarKind::Float,
                        width: 4,
                    },
                },
                Span::UNDEFINED,
            );
        }
        let (size, width, kind) = match self {
            VertexAttributeFormat::Float1 => unreachable!(),
            VertexAttributeFormat::Float2 => (VectorSize::Bi, 4, ScalarKind::Float),
            VertexAttributeFormat::Float3 => (VectorSize::Tri, 4, ScalarKind::Float),
            VertexAttributeFormat::Float4 => (VectorSize::Quad, 4, ScalarKind::Float),
            // The conversion is done by wgpu, since we specify
            // `wgpu::VertexFormat::Unorm8x4` in `CurrentPipeline::rebuild_pipeline`
            VertexAttributeFormat::Bytes4 => (VectorSize::Quad, 4, ScalarKind::Float),
        };

        module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector { size, kind, width },
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
            // This does 'vec3f(my_vec2, 0.0, 1.0)
            VertexAttributeFormat::Float2 => {
                let mut components = vec![];
                for i in 0..2 {
                    components.push(builder.evaluate_expr(Expression::AccessIndex {
                        base: base_expr,
                        index: i,
                    }));
                }
                let constant_zero = builder.module.constants.append(
                    Constant {
                        name: None,
                        specialization: None,
                        inner: ConstantInner::Scalar {
                            width: 4,
                            value: ScalarValue::Float(0.0),
                        },
                    },
                    Span::UNDEFINED,
                );
                components.push(
                    builder
                        .func
                        .expressions
                        .append(Expression::Constant(constant_zero), Span::UNDEFINED),
                );
                let constant_one = builder.module.constants.append(
                    Constant {
                        name: None,
                        specialization: None,
                        inner: ConstantInner::Scalar {
                            width: 4,
                            value: ScalarValue::Float(1.0),
                        },
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
            // This does 'vec4f(my_vec3, 1.0)'
            VertexAttributeFormat::Float3 => {
                let expr = base_expr;
                let mut components = vec![];
                for i in 0..3 {
                    components.push(builder.evaluate_expr(Expression::AccessIndex {
                        base: expr,
                        index: i,
                    }));
                }
                let constant = builder.module.constants.append(
                    Constant {
                        name: None,
                        specialization: None,
                        inner: ConstantInner::Scalar {
                            width: 4,
                            value: ScalarValue::Float(1.0),
                        },
                    },
                    Span::UNDEFINED,
                );
                components.push(
                    builder
                        .func
                        .expressions
                        .append(Expression::Constant(constant), Span::UNDEFINED),
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
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unsupported conversion from {self:?} to float4",
                )))
            }
        })
    }
}

/// Combines information extracted from the AGAL bytecode itself
/// with information provided from the AVM side of ruffle
/// (based on the Context3D methods that ActionSCript called)
#[derive(Debug)]
pub struct ShaderConfig<'a> {
    pub shader_type: ShaderType,
    pub vertex_attributes: &'a [Option<VertexAttributeFormat>; 8],
    pub version: AgalVersion,
}

#[derive(Debug)]
pub enum AgalVersion {
    Agal1,
    Agal2,
}

impl<'a> NagaBuilder<'a> {
    pub fn process_agal(
        mut agal: &[u8],
        vertex_attributes: &[Option<VertexAttributeFormat>; MAX_VERTEX_ATTRIBUTES],
    ) -> Result<Module> {
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

        let mut builder = NagaBuilder::new(ShaderConfig {
            shader_type,
            vertex_attributes,
            version,
        });

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

            builder.process_opcode(&opcode, &dest, &source1, &source2)?;
        }
        builder.finish()
    }

    // Evaluates a binary operation. The AGAL assembly should always emit a swizzle that only uses
    // a single component, so we can use any component of the source expressions.
    fn first_components_binary_op(
        &mut self,
        left: &SourceField,
        right: &SourceField,
        op: BinaryOperator,
    ) -> Result<Handle<Expression>> {
        if ![SWIZZLE_XXXX, SWIZZLE_YYYY, SWIZZLE_ZZZZ, SWIZZLE_WWWW].contains(&left.swizzle) {
            panic!(
                "LHS swizzle involved multiple distinct components for binary op {:?}: {:?}",
                op, left
            );
        }

        if ![SWIZZLE_XXXX, SWIZZLE_YYYY, SWIZZLE_ZZZZ, SWIZZLE_WWWW].contains(&right.swizzle) {
            panic!(
                "RHS swizzle involved multiple distinct components for binary op {:?}: {:?}",
                op, left
            );
        }

        let left = self.emit_source_field_load(left, false)?;
        let right = self.emit_source_field_load(right, false)?;

        let left_first_component = self.evaluate_expr(Expression::AccessIndex {
            base: left,
            index: 0,
        });

        let right_first_component = self.evaluate_expr(Expression::AccessIndex {
            base: right,
            index: 0,
        });

        let res = self.evaluate_expr(Expression::Binary {
            op,
            left: left_first_component,
            right: right_first_component,
        });

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

        let matrix4x4f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: VectorSize::Quad,
                    rows: VectorSize::Quad,
                    width: 4,
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

        let num_const_registers = module.constants.append(
            Constant {
                name: None,
                specialization: None,
                inner: ConstantInner::Scalar {
                    width: 4,
                    value: ScalarValue::Uint(match shader_config.shader_type {
                        ShaderType::Vertex => VERTEX_PROGRAM_CONTANTS,
                        ShaderType::Fragment => FRAGMENT_PROGRAM_CONSTANTS,
                    }),
                },
            },
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
                            size: ArraySize::Constant(num_const_registers),
                            stride: std::mem::size_of::<f32>() as u32 * 4,
                        },
                    },
                    Span::UNDEFINED,
                ),
                init: None,
            },
            Span::UNDEFINED,
        );

        let texture_samplers = if let ShaderType::Fragment = shader_config.shader_type {
            let samplers = (0..4)
                .map(|i| {
                    let var = module.global_variables.append(
                        GlobalVariable {
                            name: Some(format!("sampler{}", i)),
                            space: naga::AddressSpace::Handle,
                            binding: Some(naga::ResourceBinding {
                                group: 0,
                                binding: TEXTURE_SAMPLER_START_BIND_INDEX + i,
                            }),
                            ty: module.types.insert(
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
                    func.expressions
                        .append(Expression::GlobalVariable(var), Span::UNDEFINED)
                })
                .collect::<Vec<_>>();
            Some(TextureSamplers {
                clamp_linear: samplers[SAMPLER_CLAMP_LINEAR],
                clamp_nearest: samplers[SAMPLER_CLAMP_NEAREST],
                repeat_linear: samplers[SAMPLER_REPEAT_LINEAR],
                repeat_nearest: samplers[SAMPLER_REPEAT_NEAREST],
            })
        } else {
            None
        };

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
            varying_pointers: vec![],
            return_type,
            matrix4x4f,
            vec4f,
            constant_registers,
            texture_samplers,
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

    fn get_varying_pointer(&mut self, index: usize) -> Result<Handle<Expression>> {
        if index >= self.varying_pointers.len() {
            self.varying_pointers.resize(index + 1, None);
        }

        if self.varying_pointers[index].is_none() {
            match self.shader_config.shader_type {
                ShaderType::Vertex => {
                    // We can write to varying variables in the vertex shader,
                    // and the fragment shader will receive them is input.
                    // Therefore, we create a local variable for each varying,
                    // and return them at the end of the function.
                    let local = self.func.local_variables.append(
                        LocalVariable {
                            name: Some(format!("varying_{index}")),
                            ty: self.vec4f,
                            init: None,
                        },
                        Span::UNDEFINED,
                    );

                    let expr = self
                        .func
                        .expressions
                        .append(Expression::LocalVariable(local), Span::UNDEFINED);
                    let _range = self
                        .func
                        .expressions
                        .range_from(self.func.expressions.len() - 1);

                    if let TypeInner::Struct { members, .. } = &mut self.return_type.inner {
                        members.push(StructMember {
                            name: Some(format!("varying_{index}")),
                            ty: self.vec4f,
                            binding: Some(Binding::Location {
                                location: index as u32,
                                interpolation: Some(naga::Interpolation::Perspective),
                                sampling: None,
                            }),
                            offset: 0,
                        });
                    } else {
                        unreachable!();
                    }

                    self.varying_pointers[index] = Some(expr);
                }
                ShaderType::Fragment => {
                    // Function arguments might not be in the same order as the
                    // corresponding binding indices (e.g. the first argument might have binding '2').
                    // However, we only access the `FunctionArgument` expression through the `varying_pointers`
                    // vec, which is indexed by the binding index.
                    self.func.arguments.push(FunctionArgument {
                        name: None,
                        ty: self.vec4f,
                        binding: Some(Binding::Location {
                            location: index as u32,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                    });
                    let arg_index = self.func.arguments.len() - 1;

                    let expr = self.func.expressions.append(
                        Expression::FunctionArgument(arg_index as u32),
                        Span::UNDEFINED,
                    );
                    self.varying_pointers[index] = Some(expr);
                }
            };
        };

        Ok(self.varying_pointers[index].unwrap())
    }

    fn emit_const_register_load(&mut self, index: usize) -> Result<Handle<Expression>> {
        let index_const = self.module.constants.append(
            Constant {
                name: None,
                specialization: None,
                inner: ConstantInner::Scalar {
                    width: 4,
                    value: ScalarValue::Uint(index as u64),
                },
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

    fn emit_varying_load(&mut self, index: usize) -> Result<Handle<Expression>> {
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
    ) -> Result<Handle<Expression>> {
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
            self.texture_bindings[index] = Some(
                self.func
                    .expressions
                    .append(Expression::GlobalVariable(global_var), Span::UNDEFINED),
            );
        }
        Ok(self.texture_bindings[index].unwrap())
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
        let (mut base_expr, source_type) = match source.register_type {
            // We can use a function argument directly - we don't need
            // a separate Expression::Load
            RegisterType::Attribute => (
                self.get_vertex_input(source.reg_num as usize)?,
                self.shader_config.vertex_attributes[source.reg_num as usize]
                    .ok_or(Error::MissingVertexAttributeData(source.reg_num as usize))?,
            ),
            RegisterType::Varying => (
                self.emit_varying_load(source.reg_num as usize)?,
                VertexAttributeFormat::Float4,
            ),
            RegisterType::Constant => (
                self.emit_const_register_load(source.reg_num as usize)?,
                // Constants are always a vec4<f32>
                VertexAttributeFormat::Float4,
            ),
            RegisterType::Temporary => {
                let temp = self.get_temporary_register(source.reg_num as usize)?;
                (
                    self.evaluate_expr(Expression::Load { pointer: temp }),
                    VertexAttributeFormat::Float4,
                )
            }
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unimplemented source reg type {:?}",
                    source.register_type
                )))
            }
        };

        if matches!(source.direct_mode, DirectMode::Indirect) {
            return Err(Error::Unimplemented(
                "Indirect addressing not implemented".to_string(),
            ));
        }

        if extend_to_vec4 && source_type != VertexAttributeFormat::Float4 {
            base_expr = source_type.extend_to_float4(base_expr, self)?;
        }

        // This is a no-op swizzle - we can just return the base expression
        if source.swizzle == SWIZZLE_XYZW {
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

        // Optimization - use a Store instead of writing individual fields
        // when we're writing to the entire output register.
        if dest.write_mask.is_all() {
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
                    let source_component = if scalar_write {
                        // TODO - ideally, Naga would be able to tell us this information.
                        let source_is_scalar = matches!(
                            self.func.expressions[expr],
                            Expression::Math {
                                fun: MathFunction::Dot,
                                ..
                            } | Expression::As { .. }
                        );

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
    fn evaluate_expr(&mut self, expr: Expression) -> Handle<Expression> {
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
        // On the ActionScript side, the user might have specified something *other* than
        // vec4f. In that case, we need to extend the source to a vec4f if we're writing to
        // a vec4f register.
        // FIXME - do we need to do this extension in other cases?
        let do_extend = matches!(
            dest.register_type,
            RegisterType::Output | RegisterType::Varying
        );

        match opcode {
            // Copy the source register to the destination register
            Opcode::Mov => {
                let source = self.emit_source_field_load(source1, do_extend)?;
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
            // Perform 'M * v', where M is a 4x4 matrix, and 'v' is a column vector.
            Opcode::M44 => {
                let source2 = match source2 {
                    Source2::SourceField(source2) => source2,
                    _ => unreachable!(),
                };

                // Read each row of the matrix
                let source2_row0 = self.emit_source_field_load(source2, false)?;
                let source2_row1 = self.emit_source_field_load(
                    &SourceField {
                        reg_num: source2.reg_num + 1,
                        ..source2.clone()
                    },
                    false,
                )?;
                let source2_row2 = self.emit_source_field_load(
                    &SourceField {
                        reg_num: source2.reg_num + 2,
                        ..source2.clone()
                    },
                    false,
                )?;
                let source2_row3 = self.emit_source_field_load(
                    &SourceField {
                        reg_num: source2.reg_num + 3,
                        ..source2.clone()
                    },
                    false,
                )?;

                // FIXME - The naga spv backend hits an 'unreachable!'
                // if we don't create a Statement::Emit for each of these,
                // even though validation passes. We should investigate this
                // and report it upstream.
                let matrix = self.evaluate_expr(Expression::Compose {
                    ty: self.matrix4x4f,
                    components: vec![source2_row0, source2_row1, source2_row2, source2_row3],
                });

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

                let vector = self.emit_source_field_load(source1, true)?;

                let multiply = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Multiply,
                    left: matrix,
                    right: vector,
                });

                self.emit_dest_store(dest, multiply)?;
            }
            Opcode::Tex => {
                let sampler_field = source2.assert_sampler();

                let texture_samplers = self.texture_samplers.as_ref().unwrap();

                let sampler_binding = match (sampler_field.filter, sampler_field.wrapping) {
                    (Filter::Linear, Wrapping::Clamp) => texture_samplers.clamp_linear,
                    (Filter::Linear, Wrapping::Repeat) => texture_samplers.repeat_linear,
                    (Filter::Nearest, Wrapping::Clamp) => texture_samplers.clamp_nearest,
                    (Filter::Nearest, Wrapping::Repeat) => texture_samplers.repeat_nearest,
                };

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

                let image = self.emit_texture_load(texture_id as usize, sampler_field.dimension)?;
                let tex = self.evaluate_expr(Expression::ImageSample {
                    image,
                    sampler: sampler_binding,
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
                let source = self.emit_source_field_load(source1, do_extend)?;
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
                let source = self.emit_source_field_load(source1, do_extend)?;
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
                let source1 = self.emit_source_field_load(source1, do_extend)?;
                let source2 =
                    self.emit_source_field_load(source2.assert_source_field(), do_extend)?;
                let add = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Add,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, add)?;
            }
            Opcode::Sub => {
                let source1 = self.emit_source_field_load(source1, do_extend)?;
                let source2 =
                    self.emit_source_field_load(source2.assert_source_field(), do_extend)?;
                let sub = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Subtract,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, sub)?;
            }
            Opcode::Div => {
                let source1 = self.emit_source_field_load(source1, do_extend)?;
                let source2 =
                    self.emit_source_field_load(source2.assert_source_field(), do_extend)?;
                let div = self.evaluate_expr(Expression::Binary {
                    op: BinaryOperator::Divide,
                    left: source1,
                    right: source2,
                });
                self.emit_dest_store(dest, div)?;
            }
            Opcode::Max => {
                let source1 = self.emit_source_field_load(source1, do_extend)?;
                let source2 =
                    self.emit_source_field_load(source2.assert_source_field(), do_extend)?;
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
                let source = self.emit_source_field_load_with_swizzle_out(
                    source1,
                    do_extend,
                    VectorSize::Tri,
                )?;
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
                let source = self.emit_source_field_load(source1, do_extend)?;
                let rcp = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Inverse,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, rcp)?;
            }
            Opcode::Sqt => {
                let source = self.emit_source_field_load(source1, do_extend)?;
                let sqt = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Sqrt,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, sqt)?;
            }
            Opcode::Crs => {
                let source1 =
                    self.emit_source_field_load_with_swizzle_out(source1, false, VectorSize::Tri)?;
                let source2 = self.emit_source_field_load_with_swizzle_out(
                    source2.assert_source_field(),
                    false,
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
                let source1 = self.emit_source_field_load(source1, do_extend)?;
                let source2 =
                    self.emit_source_field_load(source2.assert_source_field(), do_extend)?;
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
                let source = self.emit_source_field_load(source1, do_extend)?;
                let neg = self.evaluate_expr(Expression::Unary {
                    op: UnaryOperator::Negate,
                    expr: source,
                });
                self.emit_dest_store(dest, neg)?;
            }
            Opcode::Slt => {
                let result = self.first_components_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::Less,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Seq => {
                let result = self.first_components_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::Equal,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Sne => {
                let result = self.first_components_binary_op(
                    source1,
                    source2.assert_source_field(),
                    BinaryOperator::NotEqual,
                )?;
                self.emit_dest_store(dest, result)?;
            }
            Opcode::Sat => {
                let source = self.emit_source_field_load(source1, do_extend)?;
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
                let source = self.emit_source_field_load(source1, do_extend)?;
                let frc = self.evaluate_expr(Expression::Math {
                    fun: MathFunction::Fract,
                    arg: source,
                    arg1: None,
                    arg2: None,
                    arg3: None,
                });
                self.emit_dest_store(dest, frc)?;
            }
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unimplemented opcode: {opcode:?}",
                )))
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Module> {
        // Load the 'main' output (a position or color) from our temporary location.
        let dest_load = self.evaluate_expr(Expression::Load { pointer: self.dest });
        let mut components = vec![dest_load];

        // If the vertex shader wrote to any varying registers, we need to
        // return them as well.
        if let ShaderType::Vertex = self.shader_config.shader_type {
            for i in 0..self.varying_pointers.len() {
                if self.varying_pointers[i].is_some() {
                    components.push(self.emit_varying_load(i)?);
                }
            }
        }

        // We're consuming 'self', so just store store garbage here so that we can continue
        // to use methods on 'self'
        let return_ty = std::mem::replace(
            &mut self.return_type,
            Type {
                name: None,
                inner: TypeInner::Scalar {
                    kind: ScalarKind::Float,
                    width: 0,
                },
            },
        );

        // Finalize the return type, and do emit the actual return
        let return_ty = self.module.types.insert(return_ty, Span::UNDEFINED);
        self.func.result = Some(FunctionResult {
            ty: return_ty,
            binding: None,
        });

        let return_expr = self.evaluate_expr(Expression::Compose {
            ty: return_ty,
            components,
        });

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
