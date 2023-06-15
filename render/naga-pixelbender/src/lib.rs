use std::sync::OnceLock;

use anyhow::Result;
use naga::{
    valid::{Capabilities, ValidationFlags, Validator},
    AddressSpace, ArraySize, BinaryOperator, Binding, Block, BuiltIn, Constant, ConstantInner,
    EntryPoint, Expression, Function, FunctionArgument, FunctionResult, GlobalVariable, Handle,
    ImageClass, ImageDimension, ImageQuery, LocalVariable, MathFunction, Module, ResourceBinding,
    ScalarKind, ScalarValue, ShaderStage, Span, Statement, SwizzleComponent, Type, TypeInner,
};
use ruffle_render::pixel_bender::{
    Opcode, Operation, PixelBenderParam, PixelBenderParamQualifier, PixelBenderReg,
    PixelBenderRegChannel, PixelBenderRegKind, PixelBenderShader, PixelBenderTypeOpcode,
    OUT_COORD_NAME,
};

/// The entrypoint name for the vertex and fragment shaders.
pub const SHADER_ENTRYPOINT: &str = "main";

pub struct NagaModules {
    pub vertex: naga::Module,
    pub fragment: naga::Module,

    pub float_parameters_buffer_size: u64,
    pub int_parameters_buffer_size: u64,
}

pub struct ShaderBuilder<'a> {
    module: Module,
    func: Function,
    shader: &'a PixelBenderShader,

    vec2f: Handle<Type>,
    vec4f: Handle<Type>,
    vec4i: Handle<Type>,
    image2d: Handle<Type>,
    sampler: Handle<Type>,

    clamp_nearest: Handle<Expression>,
    clamp_linear: Handle<Expression>,
    // FIXME - implement the corresponding opcode 'Sample'
    #[allow(dead_code)]
    clamp_bilinear: Handle<Expression>,

    textures: Vec<Option<Handle<Expression>>>,

    // Whenever we read from a particular register
    // for the first time, we create a new local variable
    // and store an expression here. All registers are of type vec4f
    // for simplicity. When we write to a destination register, we only
    // update the components specified in the destination write mask
    float_registers: Vec<Option<Handle<Expression>>>,

    /// Like float_registesr but with vec4i
    int_registers: Vec<Option<Handle<Expression>>>,

    // A stack of if/else blocks, using to push statements
    // into the correct block.
    blocks: Vec<BlockStackEntry>,
}

/// Handles 'if' and 'else' blocks in PixelBender bytecode.
/// When we encounter an 'OpIf' opcode, we push an `IfElse` entry onto the block stack.
/// Any subsequent opcodes will be added to the `after_if` block.
/// When we encounter an 'OpElse' opcode, we switch to adding opcodes to the `after_else` block
/// by setting `in_after_if` to false.
/// When we encouter an `OpEndIf` opcode, we pop our `IfElse` entry from the stack, and emit
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

const TEXTURE_SAMPLER_START_BIND_INDEX: u32 = 0;

// FIXME - this shouldn't actually be clamp - https://www.mcjones.org/paul/PixelBenderReference.pdf
// says that coordinates outside the range are 'transparent black'
pub const SAMPLER_CLAMP_NEAREST: u32 = 0;
pub const SAMPLER_CLAMP_LINEAR: u32 = 1;
pub const SAMPLER_CLAMP_BILINEAR: u32 = 2;

pub const SHADER_FLOAT_PARAMETERS_INDEX: u32 = 3;
// This covers ints and bool parameters
pub const SHADER_INT_PARAMETERS_INDEX: u32 = 4;

pub const TEXTURE_START_BIND_INDEX: u32 = 5;

const VERTEX_SHADER_WGSL: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    // Map coordinates from [0, 1] to [-1, 1]
    return VertexOutput(vec4<f32>((position * vec2(2.0, 2.0)) - vec2(1.0, 1.0), 0.0, 1.0));
};
"#;

impl<'a> ShaderBuilder<'a> {
    pub fn build(shader: &PixelBenderShader) -> Result<NagaModules> {
        let mut module = Module::default();

        static VERTEX_SHADER: OnceLock<Module> = OnceLock::new();
        let vertex_shader = VERTEX_SHADER
            .get_or_init(|| {
                naga::front::wgsl::Frontend::new()
                    .parse(VERTEX_SHADER_WGSL)
                    .expect("Failed to parse vertex shader")
            })
            .clone();

        let vec2f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Bi,
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
            Span::UNDEFINED,
        );

        let vec4f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Quad,
                    kind: ScalarKind::Float,
                    width: 4,
                },
            },
            Span::UNDEFINED,
        );

        let vec4i = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Quad,
                    kind: ScalarKind::Sint,
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

        let sampler = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Sampler { comparison: false },
            },
            Span::UNDEFINED,
        );

        let mut func = Function::default();
        func.arguments.push(FunctionArgument {
            name: None,
            ty: vec4f,
            binding: Some(Binding::BuiltIn(BuiltIn::Position { invariant: false })),
        });

        func.result = Some(FunctionResult {
            ty: vec4f,
            binding: Some(Binding::Location {
                location: 0,
                interpolation: None,
                sampling: None,
            }),
        });

        let samplers = (0..3)
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

        let mut builder = ShaderBuilder {
            module,
            func,
            vec2f,
            vec4f,
            vec4i,
            image2d,
            sampler,
            clamp_nearest: samplers[SAMPLER_CLAMP_NEAREST as usize],
            clamp_linear: samplers[SAMPLER_CLAMP_LINEAR as usize],
            clamp_bilinear: samplers[SAMPLER_CLAMP_BILINEAR as usize],

            shader,
            textures: Vec::new(),
            float_registers: Vec::new(),
            int_registers: Vec::new(),
            blocks: vec![BlockStackEntry::Normal(Block::new())],
        };

        let wrapper_func = builder.make_sampler_wrapper();

        let (float_parameters_buffer_size, int_parameters_buffer_size) = builder.add_arguments()?;
        builder.process_opcodes(wrapper_func)?;

        let dst = shader
            .params
            .iter()
            .find_map(|p| {
                if let PixelBenderParam::Normal {
                    qualifier: PixelBenderParamQualifier::Output,
                    reg,
                    ..
                } = p
                {
                    Some(reg)
                } else {
                    None
                }
            })
            .expect("Missing destination register!");
        assert_eq!(
            dst.channels,
            PixelBenderRegChannel::RGBA,
            "Invalid 'dest' parameter register {dst:?}"
        );

        // We've emitted all of the opcodes into the function body, so we can now load
        // from the destination register and return it from the function.
        let dst_load = builder.load_src_register(dst)?;
        builder.push_statement(Statement::Return {
            value: Some(dst_load),
        });

        let block = match builder.blocks.pop().unwrap() {
            BlockStackEntry::Normal(block) => block,
            block => panic!("Unfinished if statement: {:?}", block),
        };

        if !builder.blocks.is_empty() {
            panic!("Unbalanced blocks: {:?}", builder.blocks);
        }
        if !builder.func.body.is_empty() {
            panic!(
                "Incorrectly wrote to function body: {:?}",
                builder.func.body
            );
        }
        builder.func.body = block;

        builder.module.entry_points.push(EntryPoint {
            name: "main".to_string(),
            stage: ShaderStage::Fragment,
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: builder.func,
        });

        Ok(NagaModules {
            vertex: vertex_shader,
            fragment: builder.module,
            float_parameters_buffer_size,
            int_parameters_buffer_size,
        })
    }

    fn add_arguments(&mut self) -> Result<(u64, u64)> {
        let mut num_floats = 0;
        let mut num_ints = 0;

        let mut param_offsets = Vec::new();

        let mut out_coord = None;

        for param in &self.shader.params {
            match param {
                PixelBenderParam::Normal {
                    qualifier: PixelBenderParamQualifier::Input,
                    param_type,
                    reg,
                    name,
                    metadata: _,
                } => {
                    if name == OUT_COORD_NAME {
                        // This is passed in through a builtin, not a uniform
                        out_coord = Some(reg);
                        continue;
                    }

                    let float_offset = num_floats;
                    let int_offset = num_ints;

                    // To meet alignment requirements, each parameter is stored as a vec4 in the constants array.
                    // Smaller types (e.g. Float, Float2, Float3) are padded with zeros.
                    let (offset, is_float) = match param_type {
                        PixelBenderTypeOpcode::TFloat
                        | PixelBenderTypeOpcode::TFloat2
                        | PixelBenderTypeOpcode::TFloat3
                        | PixelBenderTypeOpcode::TFloat4 => {
                            num_floats += 1;
                            (float_offset, true)
                        }
                        PixelBenderTypeOpcode::TInt
                        | PixelBenderTypeOpcode::TInt2
                        | PixelBenderTypeOpcode::TInt3
                        | PixelBenderTypeOpcode::TInt4 => {
                            num_ints += 1;
                            (int_offset, false)
                        }
                        PixelBenderTypeOpcode::TString => continue,
                        _ => unimplemented!("Unsupported parameter type {:?}", param_type),
                    };

                    param_offsets.push((reg, offset, is_float));
                }
                PixelBenderParam::Texture {
                    index,
                    channels: _,
                    name: _,
                } => {
                    let index = *index as usize;
                    let global_var = self.module.global_variables.append(
                        GlobalVariable {
                            name: Some(format!("texture{}", index)),
                            space: AddressSpace::Handle,
                            binding: Some(ResourceBinding {
                                group: 0,
                                binding: TEXTURE_START_BIND_INDEX + index as u32,
                            }),
                            ty: self.image2d,
                            init: None,
                        },
                        Span::UNDEFINED,
                    );

                    if index >= self.textures.len() {
                        self.textures.resize(index + 1, None);
                    }
                    self.textures[index] = Some(
                        self.func
                            .expressions
                            .append(Expression::GlobalVariable(global_var), Span::UNDEFINED),
                    );
                }
                _ => {}
            }
        }

        // These globals must have at least one entry in the array to satisfy naga,
        // even if we don't have any parameters of that type.

        let num_floats_constant = self.module.constants.append(
            Constant {
                name: None,
                specialization: None,
                inner: naga::ConstantInner::Scalar {
                    width: 4,
                    value: naga::ScalarValue::Uint(num_floats.max(1) as u64),
                },
            },
            Span::UNDEFINED,
        );

        let num_ints_constant = self.module.constants.append(
            Constant {
                name: None,
                specialization: None,
                inner: naga::ConstantInner::Scalar {
                    width: 4,
                    value: naga::ScalarValue::Uint(num_ints.max(1) as u64),
                },
            },
            Span::UNDEFINED,
        );

        let shader_float_parameters = self.module.global_variables.append(
            GlobalVariable {
                name: Some("shader_float_parameters".to_string()),
                space: naga::AddressSpace::Uniform,
                binding: Some(naga::ResourceBinding {
                    group: 0,
                    binding: SHADER_FLOAT_PARAMETERS_INDEX,
                }),
                ty: self.module.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Array {
                            base: self.vec4f,
                            size: ArraySize::Constant(num_floats_constant),
                            stride: std::mem::size_of::<f32>() as u32 * 4,
                        },
                    },
                    Span::UNDEFINED,
                ),
                init: None,
            },
            Span::UNDEFINED,
        );

        let shader_int_parameters = self.module.global_variables.append(
            GlobalVariable {
                name: Some("shader_int_parameters".to_string()),
                space: naga::AddressSpace::Uniform,
                binding: Some(naga::ResourceBinding {
                    group: 0,
                    binding: SHADER_INT_PARAMETERS_INDEX,
                }),
                ty: self.module.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Array {
                            base: self.vec4i,
                            size: ArraySize::Constant(num_ints_constant),
                            stride: std::mem::size_of::<i32>() as u32 * 4,
                        },
                    },
                    Span::UNDEFINED,
                ),
                init: None,
            },
            Span::UNDEFINED,
        );

        for (reg, offset, is_float) in param_offsets {
            let global = if is_float {
                shader_float_parameters
            } else {
                shader_int_parameters
            };

            let global_base = self
                .func
                .expressions
                .append(Expression::GlobalVariable(global), Span::UNDEFINED);

            let src_ptr = self.evaluate_expr(Expression::AccessIndex {
                base: global_base,
                index: offset,
            });

            let src = self.evaluate_expr(Expression::Load { pointer: src_ptr });

            self.emit_dest_store(src, reg);
        }

        // Emit this after all other registers have been initialized
        // (it may use te same register as another parameter, but with different components)

        if let Some(coord_reg) = out_coord {
            let coord_val = self
                .func
                .expressions
                .append(Expression::FunctionArgument(0), Span::UNDEFINED);
            self.emit_dest_store(coord_val, coord_reg);
        }

        Ok((
            num_floats.max(1) as u64 * 4 * std::mem::size_of::<f32>() as u64,
            num_ints.max(1) as u64 * 4 * std::mem::size_of::<i32>() as u64,
        ))
    }

    // Works around wgpu requiring naga's strict level of uniformity analysis
    // See https://github.com/gpuweb/gpuweb/issues/3479#issuecomment-1519140312
    fn make_sampler_wrapper(&mut self) -> Handle<Function> {
        let mut func = Function {
            name: Some("sampler_wrapper".to_string()),
            arguments: vec![
                FunctionArgument {
                    name: Some("image".to_string()),
                    ty: self.image2d,
                    binding: None,
                },
                FunctionArgument {
                    name: Some("sampler".to_string()),
                    ty: self.sampler,
                    binding: None,
                },
                FunctionArgument {
                    name: Some("coord".to_string()),
                    ty: self.vec2f,
                    binding: None,
                },
            ],
            result: Some(FunctionResult {
                ty: self.vec4f,
                binding: None,
            }),
            ..Default::default()
        };

        let image = func
            .expressions
            .append(Expression::FunctionArgument(0), Span::UNDEFINED);
        let sampler = func
            .expressions
            .append(Expression::FunctionArgument(1), Span::UNDEFINED);
        let coordinate = func
            .expressions
            .append(Expression::FunctionArgument(2), Span::UNDEFINED);

        let sample = func.expressions.append(
            Expression::ImageSample {
                image,
                sampler,
                coordinate,
                array_index: None,
                offset: None,
                level: naga::SampleLevel::Auto,
                depth_ref: None,
                gather: None,
            },
            Span::UNDEFINED,
        );

        func.body.push(
            Statement::Emit(func.expressions.range_from(func.expressions.len() - 1)),
            Span::UNDEFINED,
        );

        func.body.push(
            Statement::Return {
                value: Some(sample),
            },
            Span::UNDEFINED,
        );
        self.module.functions.append(func, Span::UNDEFINED)
    }

    fn process_opcodes(&mut self, sample_wrapper_func: Handle<Function>) -> Result<()> {
        for op in &self.shader.operations {
            match op {
                Operation::Normal { opcode, dst, src } => {
                    let src = self.load_src_register(src)?;
                    let mut dst = dst.clone();
                    let evaluated = match opcode {
                        Opcode::Mov => src,
                        Opcode::Rcp => {
                            let const_one = self.module.constants.append(
                                Constant {
                                    name: None,
                                    specialization: None,
                                    inner: naga::ConstantInner::Scalar {
                                        width: 4,
                                        value: naga::ScalarValue::Float(1.0),
                                    },
                                },
                                Span::UNDEFINED,
                            );
                            let expr_one = self
                                .func
                                .expressions
                                .append(Expression::Constant(const_one), Span::UNDEFINED);

                            let vec_one = self.evaluate_expr(Expression::Splat {
                                size: naga::VectorSize::Quad,
                                value: expr_one,
                            });

                            // Perform 'vec4(1.0, 1.0, 1.0. 1.0) / src'
                            self.evaluate_expr(Expression::Binary {
                                op: BinaryOperator::Divide,
                                left: vec_one,
                                right: src,
                            })
                        }
                        Opcode::Sub | Opcode::Add | Opcode::Mul => {
                            // The destiation is also used as the first operand: 'dst = dst <op> src'
                            let left = self.load_src_register(&dst)?;

                            let op = match opcode {
                                Opcode::Sub => BinaryOperator::Subtract,
                                Opcode::Add => BinaryOperator::Add,
                                Opcode::Mul => BinaryOperator::Multiply,
                                _ => unreachable!(),
                            };

                            self.evaluate_expr(Expression::Binary {
                                op,
                                left,
                                right: src,
                            })
                        }
                        Opcode::LessThan => {
                            let left = self.load_src_register(&dst)?;
                            let res = self.evaluate_expr(Expression::Binary {
                                op: BinaryOperator::Less,
                                left,
                                right: src,
                            });

                            // The 'LessThan' opcodes appears to compare the src and dst, and then
                            // write the result to the 'R' component of int register 0
                            dst = PixelBenderReg {
                                index: 0,
                                channels: vec![PixelBenderRegChannel::R],
                                kind: PixelBenderRegKind::Int,
                            };
                            // We get back a vec of bools from BinaryOperator::Less, so convert it to a vec of floats
                            self.evaluate_expr(Expression::As {
                                expr: res,
                                kind: ScalarKind::Float,
                                convert: Some(4),
                            })
                        }
                        Opcode::LogicalOr => {
                            // The destiation is also used as the first operand: 'dst = dst - src'
                            let left = self.load_src_register(&dst)?;
                            let left_bool = self.evaluate_expr(Expression::As {
                                expr: left,
                                kind: ScalarKind::Bool,
                                convert: Some(1),
                            });
                            let right_bool = self.evaluate_expr(Expression::As {
                                expr: src,
                                kind: ScalarKind::Bool,
                                convert: Some(1),
                            });

                            // Note - this should just be a `LogicalOr` between two vectors.
                            // However, Naga currently handles this incorrectly - see https://github.com/gfx-rs/naga/issues/1931
                            // For now, work around this by manually applying it component-wise.

                            let source_components: Vec<_> = (0..4)
                                .map(|index| {
                                    self.evaluate_expr(Expression::AccessIndex {
                                        base: left_bool,
                                        index,
                                    })
                                })
                                .collect();

                            let dest_components: Vec<_> = (0..4)
                                .map(|index| {
                                    self.evaluate_expr(Expression::AccessIndex {
                                        base: right_bool,
                                        index,
                                    })
                                })
                                .collect();

                            let res_components = (0..4)
                                .map(|index| {
                                    let component_or = self.evaluate_expr(Expression::Binary {
                                        op: BinaryOperator::LogicalOr,
                                        left: source_components[index],
                                        right: dest_components[index],
                                    });

                                    // We get back a bool from BinaryOperator::LogicalOr, so convert it to a float
                                    self.evaluate_expr(Expression::As {
                                        expr: component_or,
                                        kind: ScalarKind::Float,
                                        convert: Some(4),
                                    })
                                })
                                .collect();

                            self.evaluate_expr(Expression::Compose {
                                ty: self.vec4f,
                                components: res_components,
                            })
                        }
                        Opcode::Floor => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Floor,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Length => {
                            let length = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Length,
                                arg: src,
                                arg1: None,
                                arg2: None,
                                arg3: None,
                            });
                            self.evaluate_expr(Expression::Splat {
                                size: naga::VectorSize::Quad,
                                value: length,
                            })
                        }
                        _ => {
                            unimplemented!("Unimplemented opcode {opcode:?}");
                        }
                    };
                    self.emit_dest_store(evaluated, &dst);
                }
                Operation::SampleLinear { dst, src, tf }
                | Operation::SampleNearest { dst, src, tf } => {
                    let mut coord = self.load_src_register(src)?;
                    coord = self.evaluate_expr(Expression::Swizzle {
                        size: naga::VectorSize::Bi,
                        vector: coord,
                        // Only the first two components matter
                        pattern: [
                            SwizzleComponent::X,
                            SwizzleComponent::Y,
                            SwizzleComponent::W,
                            SwizzleComponent::W,
                        ],
                    });

                    let size_vec = self.evaluate_expr(Expression::ImageQuery {
                        image: self.textures[*tf as usize].unwrap(),
                        query: ImageQuery::Size { level: None },
                    });

                    let size_vec_float = self.evaluate_expr(Expression::As {
                        kind: crate::ScalarKind::Float,
                        expr: size_vec,
                        convert: Some(4),
                    });

                    let normalized_coord = self.evaluate_expr(Expression::Binary {
                        op: BinaryOperator::Divide,
                        left: coord,
                        right: size_vec_float,
                    });

                    let image = self.textures[*tf as usize].unwrap();

                    let sampler = match op {
                        Operation::SampleNearest { .. } => self.clamp_nearest,
                        Operation::SampleLinear { .. } => self.clamp_linear,
                        _ => unreachable!(),
                    };

                    // Don't evaluate this expression - it gets evaluated by Statement::Call
                    let result = self
                        .func
                        .expressions
                        .append(Expression::CallResult(sample_wrapper_func), Span::UNDEFINED);

                    // Call our helper function, which just calls 'Expression::ImageSample' with
                    // the provided parameters. This works around a uniformity analysis issue
                    // with wgpu/naga
                    self.push_statement(Statement::Call {
                        function: sample_wrapper_func,
                        arguments: vec![image, sampler, normalized_coord],
                        result: Some(result),
                    });

                    self.emit_dest_store(result, dst);
                }
                Operation::LoadFloat { dst, val } => {
                    let const_val = self.module.constants.append(
                        crate::Constant {
                            name: None,
                            specialization: None,
                            inner: ConstantInner::Scalar {
                                width: 4,
                                value: ScalarValue::Float(*val as f64),
                            },
                        },
                        Span::UNDEFINED,
                    );
                    let const_expr = self
                        .func
                        .expressions
                        .append(Expression::Constant(const_val), Span::UNDEFINED);

                    let const_vec = self.evaluate_expr(Expression::Splat {
                        size: naga::VectorSize::Quad,
                        value: const_expr,
                    });
                    self.emit_dest_store(const_vec, dst);
                }
                Operation::LoadInt { dst, val } => {
                    let const_val = self.module.constants.append(
                        crate::Constant {
                            name: None,
                            specialization: None,
                            inner: ConstantInner::Scalar {
                                width: 4,
                                value: ScalarValue::Sint(*val as i64),
                            },
                        },
                        Span::UNDEFINED,
                    );
                    let const_expr = self
                        .func
                        .expressions
                        .append(Expression::Constant(const_val), Span::UNDEFINED);

                    let const_vec = self.evaluate_expr(Expression::Splat {
                        size: naga::VectorSize::Quad,
                        value: const_expr,
                    });
                    self.emit_dest_store(const_vec, dst);
                }
                Operation::If { src } => {
                    let scalar_zero = match src.kind {
                        PixelBenderRegKind::Float => ScalarValue::Float(0.0),
                        PixelBenderRegKind::Int => ScalarValue::Sint(0),
                    };
                    if src.channels.len() != 1 {
                        panic!("If condition must be a scalar: {src:?}");
                    }

                    // FIXME - `load_src_register` always gives us a vec4 - ideally, we would
                    // have a flag to avoid this pointless splat-and-extract.
                    let src = self.load_src_register(src)?;
                    let first_component = self.evaluate_expr(Expression::AccessIndex {
                        base: src,
                        index: 0,
                    });

                    let const_zero = self.module.constants.append(
                        Constant {
                            name: None,
                            specialization: None,
                            inner: ConstantInner::Scalar {
                                width: 4,
                                value: scalar_zero,
                            },
                        },
                        Span::UNDEFINED,
                    );

                    let expr_zero = self
                        .func
                        .expressions
                        .append(Expression::Constant(const_zero), Span::UNDEFINED);

                    let is_true = self.evaluate_expr(Expression::Binary {
                        op: BinaryOperator::NotEqual,
                        left: first_component,
                        right: expr_zero,
                    });

                    self.blocks.push(BlockStackEntry::IfElse {
                        after_if: Block::new(),
                        after_else: Block::new(),
                        in_after_if: true,
                        condition: is_true,
                    })
                }
                Operation::Else => {
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
                Operation::EndIf => {
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
                _ => unimplemented!("Operation {op:?} not yet implemented"),
            }
        }
        Ok(())
    }

    /// Gets a pointer to the given register - this does *not* perform a load, so it can
    /// be used with both `Expression::Load` and `Statement::Store`
    fn register_pointer(&mut self, reg: &PixelBenderReg) -> Result<Handle<Expression>> {
        let index = reg.index as usize;

        let (ty, registers, register_kind_name) = match reg.kind {
            PixelBenderRegKind::Float => (self.vec4f, &mut self.float_registers, "float"),
            PixelBenderRegKind::Int => (self.vec4i, &mut self.int_registers, "int"),
        };

        if index >= registers.len() {
            registers.resize(index + 1, None);
        }

        if registers[index].is_none() {
            let local = self.func.local_variables.append(
                LocalVariable {
                    name: Some(format!("local_{register_kind_name}_reg_{index}")),
                    ty,
                    init: None,
                },
                Span::UNDEFINED,
            );

            let expr = self
                .func
                .expressions
                .append(Expression::LocalVariable(local), Span::UNDEFINED);
            registers[index] = Some(expr);
        }
        Ok(registers[index].unwrap())
    }

    /// Loads a vec4f/vec4i from the given register. Note that all registers are 4-component
    /// vectors - if the `PixelBenderReg` requests fewer components then that, then the extra
    /// components will be meaningless. This greatly simplifies the code, since we don't need
    /// to track whether or not we have a scalar or a vector everywhere.
    fn load_src_register(&mut self, reg: &PixelBenderReg) -> Result<Handle<Expression>> {
        let reg_ptr = self.register_pointer(reg)?;
        let reg_value = self.evaluate_expr(Expression::Load { pointer: reg_ptr });

        let mut swizzle_components = reg
            .channels
            .iter()
            .map(|c| match c {
                PixelBenderRegChannel::R => SwizzleComponent::X,
                PixelBenderRegChannel::G => SwizzleComponent::Y,
                PixelBenderRegChannel::B => SwizzleComponent::Z,
                PixelBenderRegChannel::A => SwizzleComponent::W,
            })
            .collect::<Vec<_>>();

        if swizzle_components.len() < 4 {
            // Pad with W - these components will be ignored, since whatever uses
            // the result will only use the components corresponding to 'reg.channels'
            swizzle_components.resize(4, SwizzleComponent::W);
        }

        Ok(self.evaluate_expr(Expression::Swizzle {
            size: naga::VectorSize::Quad,
            vector: reg_value,
            pattern: swizzle_components.try_into().unwrap(),
        }))
    }

    /// Creates a `Statement::Emit` covering `expr`
    fn evaluate_expr(&mut self, expr: Expression) -> Handle<Expression> {
        let prev_len = self.func.expressions.len();
        let expr = self.func.expressions.append(expr, Span::UNDEFINED);
        let range = self.func.expressions.range_from(prev_len);
        self.push_statement(Statement::Emit(range));
        expr
    }

    // Emits a store of `expr` to the destination register, taking into account the store mask.
    fn emit_dest_store(&mut self, expr: Handle<Expression>, dst: &PixelBenderReg) {
        let dst_register = self.register_pointer(dst).unwrap();
        for (dst_channel, src_channel) in
            dst.channels.iter().zip(PixelBenderRegChannel::RGBA.iter())
        {
            // Write each channel of the source to the channel specified by the destiation mask
            let src_component_index = *src_channel as u32;
            let dst_component_index = *dst_channel as u32;
            let src_component = self.evaluate_expr(Expression::AccessIndex {
                base: expr,
                index: src_component_index,
            });

            let dst_component = self.evaluate_expr(Expression::AccessIndex {
                base: dst_register,
                index: dst_component_index,
            });

            let scalar_kind = match dst.kind {
                PixelBenderRegKind::Float => ScalarKind::Float,
                PixelBenderRegKind::Int => ScalarKind::Sint,
            };

            let src_cast = self.evaluate_expr(Expression::As {
                kind: scalar_kind,
                expr: src_component,
                convert: Some(4),
            });

            self.push_statement(Statement::Store {
                pointer: dst_component,
                value: src_cast,
            })
        }
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
}

#[allow(dead_code)]
fn to_wgsl(module: &naga::Module) -> String {
    let mut out = String::new();

    let mut validator = Validator::new(
        ValidationFlags::all() - ValidationFlags::CONTROL_FLOW_UNIFORMITY,
        Capabilities::all(),
    );
    let module_info = validator
        .validate(module)
        .unwrap_or_else(|e| panic!("Validation failed: {:#?}", e));

    let mut writer =
        naga::back::wgsl::Writer::new(&mut out, naga::back::wgsl::WriterFlags::EXPLICIT_TYPES);

    writer.write(module, &module_info).expect("Writing failed");
    out
}
