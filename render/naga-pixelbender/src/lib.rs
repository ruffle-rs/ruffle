use std::{num::NonZeroU32, sync::OnceLock, vec};

use anyhow::Result;
use naga::{
    valid::{Capabilities, ValidationFlags, Validator},
    AddressSpace, ArraySize, BinaryOperator, Binding, Block, BuiltIn, EntryPoint, Expression,
    Function, FunctionArgument, FunctionResult, GlobalVariable, Handle, ImageClass, ImageDimension,
    ImageQuery, Literal, LocalVariable, MathFunction, Module, RelationalFunction, ResourceBinding,
    ScalarKind, ShaderStage, Span, Statement, SwizzleComponent, Type, TypeInner, VectorSize,
};
use ruffle_render::pixel_bender::{
    Opcode, Operation, PixelBenderParam, PixelBenderParamQualifier, PixelBenderReg,
    PixelBenderRegChannel, PixelBenderRegKind, PixelBenderShader, PixelBenderTypeOpcode,
    OUT_COORD_NAME,
};

pub const VERTEX_SHADER_ENTRYPOINT: &str = "filter__vertex_entry_point";
pub const FRAGMENT_SHADER_ENTRYPOINT: &str = "main";

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
    mat2x2f: Handle<Type>,
    mat3x3f: Handle<Type>,
    mat4x4f: Handle<Type>,
    image2d: Handle<Type>,
    sampler: Handle<Type>,

    // The value 0.0f32
    zerof32: Handle<Expression>,
    // The value 0i32
    zeroi32: Handle<Expression>,
    // The value vec4f(0.0)
    zerovec4f: Handle<Expression>,
    // The value 1.0f32
    onef32: Handle<Expression>,

    // A temporary vec4f local variable.
    // Currently used during texture sampling.
    temp_vec4f_local: Handle<Expression>,

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
/// When we encounter an `OpEndIf` opcode, we pop our `IfElse` entry from the stack, and emit
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

pub const SAMPLER_CLAMP_NEAREST: u32 = 0;
pub const SAMPLER_CLAMP_LINEAR: u32 = 1;
pub const SAMPLER_CLAMP_BILINEAR: u32 = 2;

pub const SHADER_FLOAT_PARAMETERS_INDEX: u32 = 3;
// This covers ints and bool parameters
pub const SHADER_INT_PARAMETERS_INDEX: u32 = 4;

// A parameter controlling whether or not we produce transparent black (zero)
// for textures samples with out-of-range coordinates. This is a vec4f
// uniform - when it's 0.0, we use the default clamping behavior, and produce
// transparent black when it's any other value. This would ideally be a single
// f32, but web requires a minimum of 16 bytes.
//
// Note - https://www.mcjones.org/paul/PixelBenderReference.pdf
// claims that coordinates outside the range are 'transparent black'.
// However, some testing shows that the actual behavior is 'clamp' (at least
// when a shader is run through a ShaderJob, and is only 'transparent black'
// when a ShaderFilter is used. We set this uniform from Ruffle based on
// how the shader is being invoked.
pub const ZEROED_OUT_OF_RANGE_MODE_INDEX: u32 = 5;

pub const TEXTURE_START_BIND_INDEX: u32 = 6;

impl ShaderBuilder<'_> {
    pub fn build(shader: &PixelBenderShader) -> Result<NagaModules> {
        let mut module = Module::default();

        static VERTEX_SHADER: OnceLock<Module> = OnceLock::new();
        let vertex_shader = VERTEX_SHADER
            .get_or_init(|| {
                naga::front::wgsl::parse_str(ruffle_render::shader_source::SHADER_FILTER_COMMON)
                    .expect("Failed to parse vertex shader")
            })
            .clone();

        let vec2f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Bi,
                    scalar: naga::Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let vec4f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Quad,

                    scalar: naga::Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let vec4i = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size: naga::VectorSize::Quad,
                    scalar: naga::Scalar::I32,
                },
            },
            Span::UNDEFINED,
        );

        let mat2x2f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: naga::VectorSize::Bi,
                    rows: naga::VectorSize::Bi,
                    scalar: naga::Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let mat3x3f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: naga::VectorSize::Tri,
                    rows: naga::VectorSize::Tri,
                    scalar: naga::Scalar::F32,
                },
            },
            Span::UNDEFINED,
        );

        let mat4x4f = module.types.insert(
            Type {
                name: None,
                inner: TypeInner::Matrix {
                    columns: naga::VectorSize::Quad,
                    rows: naga::VectorSize::Quad,
                    scalar: naga::Scalar::F32,
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
        // UV coordinates from vertex shader - unused, but wgpu
        // requires that we consume all outputs from the vertex shader
        func.arguments.push(FunctionArgument {
            name: None,
            ty: vec2f,
            binding: Some(Binding::Location {
                location: 0,
                interpolation: Some(naga::Interpolation::Perspective),
                sampling: Some(naga::Sampling::Center),
                second_blend_source: false,
            }),
        });

        func.result = Some(FunctionResult {
            ty: vec4f,
            binding: Some(Binding::Location {
                location: 0,
                interpolation: None,
                sampling: None,
                second_blend_source: false,
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

        let zeroi32 = func
            .expressions
            .append(Expression::Literal(Literal::I32(0)), Span::UNDEFINED);

        let zerof32 = func
            .expressions
            .append(Expression::Literal(Literal::F32(0.0)), Span::UNDEFINED);

        let onef32 = func
            .expressions
            .append(Expression::Literal(Literal::F32(1.0)), Span::UNDEFINED);

        let mut blocks = vec![BlockStackEntry::Normal(Block::new())];

        let zerovec4f = evaluate_expr(
            &mut func,
            &mut blocks,
            Expression::Compose {
                ty: vec4f,
                components: vec![zerof32, zerof32, zerof32, zerof32],
            },
        );

        let temp_vec4f_local = func.local_variables.append(
            LocalVariable {
                name: Some("temp_vec4f_local".to_string()),
                ty: vec4f,
                init: None,
            },
            Span::UNDEFINED,
        );
        let temp_vec4f_local = func
            .expressions
            .append(Expression::LocalVariable(temp_vec4f_local), Span::UNDEFINED);

        let mut builder = ShaderBuilder {
            module,
            func,
            vec2f,
            vec4f,
            vec4i,
            mat2x2f,
            mat3x3f,
            mat4x4f,
            image2d,
            sampler,
            zerof32,
            zeroi32,
            zerovec4f,
            onef32,
            temp_vec4f_local,
            clamp_nearest: samplers[SAMPLER_CLAMP_NEAREST as usize],
            clamp_linear: samplers[SAMPLER_CLAMP_LINEAR as usize],
            clamp_bilinear: samplers[SAMPLER_CLAMP_BILINEAR as usize],

            shader,
            textures: Vec::new(),
            float_registers: Vec::new(),
            int_registers: Vec::new(),
            blocks,
        };

        let zeroed_out_of_range_mode_global = builder.module.global_variables.append(
            GlobalVariable {
                name: Some("zeroed_out_of_range_mode".to_string()),
                space: naga::AddressSpace::Uniform,
                binding: Some(naga::ResourceBinding {
                    group: 0,
                    binding: ZEROED_OUT_OF_RANGE_MODE_INDEX,
                }),
                ty: vec4f,
                init: None,
            },
            Span::UNDEFINED,
        );

        let zeroed_out_of_range_expr = builder.func.expressions.append(
            Expression::GlobalVariable(zeroed_out_of_range_mode_global),
            Span::UNDEFINED,
        );
        let zeroed_out_of_range_expr = builder.evaluate_expr(Expression::Load {
            pointer: zeroed_out_of_range_expr,
        });
        let zeroed_out_of_range_expr = builder.evaluate_expr(Expression::Binary {
            op: BinaryOperator::NotEqual,
            left: zeroed_out_of_range_expr,
            right: builder.zerovec4f,
        });
        let zeroed_out_of_range_expr = builder.evaluate_expr(Expression::Relational {
            fun: RelationalFunction::Any,
            argument: zeroed_out_of_range_expr,
        });

        let wrapper_func = builder.make_sampler_wrapper();

        let (float_parameters_buffer_size, int_parameters_buffer_size) = builder.add_arguments()?;
        builder.process_opcodes(wrapper_func, zeroed_out_of_range_expr)?;

        let (dst, dst_param_type) = shader
            .params
            .iter()
            .find_map(|p| {
                if let PixelBenderParam::Normal {
                    qualifier: PixelBenderParamQualifier::Output,
                    reg,
                    param_type,
                    ..
                } = p
                {
                    Some((reg, param_type))
                } else {
                    None
                }
            })
            .expect("Missing destination register!");

        let expected_dst_channels = match dst_param_type {
            PixelBenderTypeOpcode::TFloat4 => PixelBenderRegChannel::RGBA.as_slice(),
            PixelBenderTypeOpcode::TFloat3 => PixelBenderRegChannel::RGB.as_slice(),
            _ => panic!("Invalid destination register type: {:?}", dst_param_type),
        };
        assert_eq!(
            dst.channels, expected_dst_channels,
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
        let mut num_vec4fs = 0;
        let mut num_vec4is = 0;

        let mut param_offsets = Vec::new();

        let mut out_coord = None;

        enum ParamKind {
            Float,
            Int,
            FloatMatrix,
        }

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

                    let float_offset = num_vec4fs;
                    let int_offset = num_vec4is;

                    // To meet alignment requirements, each parameter is stored as some number of vec4s in the constants array.
                    // Smaller types (e.g. Float, Float2, Float3) are padded with zeros.
                    let (offset, is_float) = match param_type {
                        PixelBenderTypeOpcode::TFloat
                        | PixelBenderTypeOpcode::TFloat2
                        | PixelBenderTypeOpcode::TFloat3
                        | PixelBenderTypeOpcode::TFloat4 => {
                            num_vec4fs += 1;
                            (float_offset, ParamKind::Float)
                        }
                        PixelBenderTypeOpcode::TInt
                        | PixelBenderTypeOpcode::TInt2
                        | PixelBenderTypeOpcode::TInt3
                        | PixelBenderTypeOpcode::TInt4 => {
                            num_vec4is += 1;
                            (int_offset, ParamKind::Int)
                        }
                        PixelBenderTypeOpcode::TString => continue,
                        PixelBenderTypeOpcode::TFloat2x2 => {
                            // A 2x2 matrix fits into a single vec4
                            num_vec4fs += 1;
                            (float_offset, ParamKind::FloatMatrix)
                        }
                        PixelBenderTypeOpcode::TFloat3x3 => {
                            // Each row of the matrix is stored in a vec4 (with the last component of each set to 0)
                            num_vec4fs += 3;
                            (float_offset, ParamKind::FloatMatrix)
                        }
                        PixelBenderTypeOpcode::TFloat4x4 => {
                            // Each row of the matrix is a vec4
                            num_vec4fs += 4;
                            (float_offset, ParamKind::FloatMatrix)
                        }
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
                            size: ArraySize::Constant(NonZeroU32::new(num_vec4fs.max(1)).unwrap()),
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
                            size: ArraySize::Constant(NonZeroU32::new(num_vec4is.max(1)).unwrap()),
                            stride: std::mem::size_of::<i32>() as u32 * 4,
                        },
                    },
                    Span::UNDEFINED,
                ),
                init: None,
            },
            Span::UNDEFINED,
        );

        for (reg, offset, param_kind) in param_offsets {
            let param_global = match param_kind {
                ParamKind::Float | ParamKind::FloatMatrix => shader_float_parameters,
                ParamKind::Int => shader_int_parameters,
            };

            let global_base = self
                .func
                .expressions
                .append(Expression::GlobalVariable(param_global), Span::UNDEFINED);

            let src_ptr = self.evaluate_expr(Expression::AccessIndex {
                base: global_base,
                index: offset,
            });

            let src_expr = match reg.channels[0] {
                PixelBenderRegChannel::M2x2
                | PixelBenderRegChannel::M3x3
                | PixelBenderRegChannel::M4x4 => {
                    let get_vec_ptr = |this: &mut Self, index| {
                        Ok(this.evaluate_expr(Expression::AccessIndex {
                            base: global_base,
                            index: offset + index,
                        }))
                    };
                    self.load_reg_as_matrix(reg.channels[0], get_vec_ptr)?
                }
                _ => self.evaluate_expr(Expression::Load { pointer: src_ptr }),
            };

            self.emit_dest_store(src_expr, reg)?;
        }

        // Emit this after all other registers have been initialized
        // (it may use te same register as another parameter, but with different components)

        if let Some(coord_reg) = out_coord {
            let coord_val = self
                .func
                .expressions
                .append(Expression::FunctionArgument(0), Span::UNDEFINED);
            self.emit_dest_store(coord_val, coord_reg)?;
        }

        Ok((
            num_vec4fs.max(1) as u64 * 4 * std::mem::size_of::<f32>() as u64,
            num_vec4is.max(1) as u64 * 4 * std::mem::size_of::<i32>() as u64,
        ))
    }

    /// Samples a texture, determining the out-of-range coordinate behavior
    /// based on `zeroed_out_of_range_expr`. See the comments on `ZEROED_OUT_OF_RANGE_MODE_INDEX`
    /// for more details.
    fn sample_texture(
        &mut self,
        sample_wrapper_func: Handle<Function>,
        normalized_coord: Handle<Expression>,
        image: Handle<Expression>,
        sampler: Handle<Expression>,
        zeroed_out_of_range_expr: Handle<Expression>,
    ) -> Handle<Expression> {
        // Don't evaluate this expression - it gets evaluated by Statement::Call
        let result = self
            .func
            .expressions
            .append(Expression::CallResult(sample_wrapper_func), Span::UNDEFINED);

        // Build up the expression '(coord.x < 0.0 || coord.x > 1.0 || coord.y < 0.0 || coord.y > 1.0)'

        let x_coord: Handle<Expression> = self.evaluate_expr(Expression::AccessIndex {
            base: normalized_coord,
            index: 0,
        });

        let y_coord = self.evaluate_expr(Expression::AccessIndex {
            base: normalized_coord,
            index: 1,
        });

        let x_coord_lt_zero = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::Less,
            left: x_coord,
            right: self.zerof32,
        });

        let x_coord_gt_one = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::Greater,
            left: x_coord,
            right: self.onef32,
        });

        let y_coord_lt_zero = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::Less,
            left: y_coord,
            right: self.zerof32,
        });

        let y_coord_gt_one = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::Greater,
            left: y_coord,
            right: self.onef32,
        });

        let x_coord_logical_or = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::LogicalOr,
            left: x_coord_lt_zero,
            right: x_coord_gt_one,
        });

        let y_coord_logical_or = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::LogicalOr,
            left: y_coord_lt_zero,
            right: y_coord_gt_one,
        });

        let any_coord_out_of_range = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::LogicalOr,
            left: x_coord_logical_or,
            right: y_coord_logical_or,
        });

        let out_of_range_cond = self.evaluate_expr(Expression::Binary {
            op: BinaryOperator::LogicalAnd,
            left: zeroed_out_of_range_expr,
            right: any_coord_out_of_range,
        });

        // Construct the statements:
        // ```
        // if (zeroed_out_of_range_expr && any_coord_out_of_range) {
        //    temp_local = vec4f(0.0);
        // else {
        //    temp_local = sample_wrapper_func(image, sampler, normalized_coord);
        // }
        // return temp_local
        // ```
        //
        // Note that due to the overly restrictive uniformity analysis in wgpu/naga,
        // we need this `if/else` at every call site - it cannot be inlined into
        // `sample_wrapper_func`

        let mut good_coord_block = Block::new();
        // Call our helper function, which just calls 'Expression::ImageSample' with
        // the provided parameters. This works around a uniformity analysis issue
        // with wgpu/naga
        good_coord_block.push(
            Statement::Call {
                function: sample_wrapper_func,
                arguments: vec![image, sampler, normalized_coord],
                result: Some(result),
            },
            Span::UNDEFINED,
        );
        good_coord_block.push(
            Statement::Store {
                pointer: self.temp_vec4f_local,
                value: result,
            },
            Span::UNDEFINED,
        );

        let mut bad_coord_block = Block::new();
        let zero_vec = self.evaluate_expr(Expression::Splat {
            size: VectorSize::Quad,
            value: self.zerof32,
        });
        bad_coord_block.push(
            Statement::Store {
                pointer: self.temp_vec4f_local,
                value: zero_vec,
            },
            Span::UNDEFINED,
        );

        self.push_statement(Statement::If {
            condition: out_of_range_cond,
            accept: bad_coord_block,
            reject: good_coord_block,
        });

        self.evaluate_expr(Expression::Load {
            pointer: self.temp_vec4f_local,
        })
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

    fn process_opcodes(
        &mut self,
        sample_wrapper_func: Handle<Function>,
        zeroed_out_of_range_expr: Handle<Expression>,
    ) -> Result<()> {
        for op in &self.shader.operations {
            match op {
                Operation::Normal {
                    opcode,
                    dst,
                    src: src_reg,
                } => {
                    let src = self.load_src_register(src_reg)?;
                    let mut dst = dst.clone();
                    let evaluated = match opcode {
                        Opcode::Mov => src,
                        Opcode::Rcp => {
                            let vec_one = self.evaluate_expr(Expression::Splat {
                                size: naga::VectorSize::Quad,
                                value: self.onef32,
                            });

                            // Perform 'vec4(1.0, 1.0, 1.0. 1.0) / src'
                            self.evaluate_expr(Expression::Binary {
                                op: BinaryOperator::Divide,
                                left: vec_one,
                                right: src,
                            })
                        }
                        Opcode::Sub | Opcode::Add | Opcode::Mul => {
                            // The destination is also used as the first operand: 'dst = dst <op> src'
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
                        Opcode::LogicalOr | Opcode::LogicalAnd => {
                            // The destination is also used as the first operand: 'dst = dst || src' or 'dst = dst && src'
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

                            // Note - this should just be a `LogicalOr/LogicalAnd` between two vectors.
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

                            let binary_op = match opcode {
                                Opcode::LogicalOr => BinaryOperator::LogicalOr,
                                Opcode::LogicalAnd => BinaryOperator::LogicalAnd,
                                _ => unreachable!(),
                            };

                            let res_components = (0..4)
                                .map(|index| {
                                    let component_or = self.evaluate_expr(Expression::Binary {
                                        op: binary_op,
                                        left: source_components[index],
                                        right: dest_components[index],
                                    });

                                    // We get back a bool from BinaryOperator::LogicalOr/LogicalAnd, so convert it to a float
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
                            // Don't pad the result, as adding extra components changes the length
                            let src_val = self.load_src_register_with_padding(src_reg, false)?;
                            let length = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Length,
                                arg: src_val,
                                arg1: None,
                                arg2: None,
                                arg3: None,
                            });
                            self.evaluate_expr(Expression::Splat {
                                size: naga::VectorSize::Quad,
                                value: length,
                            })
                        }
                        Opcode::MatVecMul => {
                            let right = self.load_src_register_with_padding(&dst, false)?;
                            // This is always a vector, so no need to use `pad_result`
                            self.evaluate_expr(Expression::Binary {
                                op: BinaryOperator::Multiply,
                                left: src,
                                right,
                            })
                        }
                        Opcode::VecMatMul => {
                            let vec = self.load_src_register_with_padding(&dst, false)?;
                            // This is always a vector, so no need to use `pad_result`
                            self.evaluate_expr(Expression::Binary {
                                op: BinaryOperator::Multiply,
                                left: vec,
                                right: src,
                            })
                        }
                        Opcode::Distance => {
                            let left = self.load_src_register_with_padding(&dst, false)?;
                            let right = self.load_src_register_with_padding(src_reg, false)?;
                            let dist = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Distance,
                                arg: left,
                                arg1: Some(right),
                                arg2: None,
                                arg3: None,
                            });
                            let res = self.evaluate_expr(Expression::Splat {
                                size: VectorSize::Quad,
                                value: dist,
                            });
                            self.pad_result(res, src_reg.is_scalar())
                        }
                        Opcode::Max => {
                            let right = self.load_src_register(&dst)?;
                            self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Max,
                                arg: src,
                                arg1: Some(right),
                                arg2: None,
                                arg3: None,
                            })
                        }
                        Opcode::Min => {
                            let right = self.load_src_register(&dst)?;
                            self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Min,
                                arg: src,
                                arg1: Some(right),
                                arg2: None,
                                arg3: None,
                            })
                        }
                        Opcode::Normalize => {
                            let src = self.load_src_register_with_padding(src_reg, false)?;
                            let res = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Normalize,
                                arg: src,
                                arg1: None,
                                arg2: None,
                                arg3: None,
                            });
                            self.pad_result(res, src_reg.is_scalar())
                        }
                        Opcode::Exp => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Exp,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Pow => {
                            let dst_val = self.load_src_register(&dst)?;
                            self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Pow,
                                arg: dst_val,
                                arg1: Some(src),
                                arg2: None,
                                arg3: None,
                            })
                        }
                        Opcode::Abs => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Abs,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Sin => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Sin,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Asin => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Asin,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Cos => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Cos,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Acos => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Acos,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Tan => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Tan,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Atan => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Atan,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        Opcode::Atan2 => {
                            let dst_val = self.load_src_register(&dst)?;
                            self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Atan2,
                                arg: dst_val,
                                arg1: Some(src),
                                arg2: None,
                                arg3: None,
                            })
                        }
                        Opcode::DotProduct => {
                            let src_val: Handle<Expression> =
                                self.load_src_register_with_padding(src_reg, false)?;
                            let dst_val = self.load_src_register_with_padding(&dst, false)?;
                            let dot = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Dot,
                                arg: dst_val,
                                arg1: Some(src_val),
                                arg2: None,
                                arg3: None,
                            });
                            self.evaluate_expr(Expression::Splat {
                                size: VectorSize::Quad,
                                value: dot,
                            })
                        }
                        Opcode::Sqrt => {
                            let src_val = self.load_src_register_with_padding(src_reg, false)?;
                            let res = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Sqrt,
                                arg: src_val,
                                arg1: None,
                                arg2: None,
                                arg3: None,
                            });
                            self.pad_result(res, src_reg.is_scalar())
                        }
                        Opcode::Equal
                        | Opcode::NotEqual
                        | Opcode::LessThan
                        | Opcode::LessThanEqual => {
                            let bin_op: BinaryOperator = match opcode {
                                Opcode::Equal => BinaryOperator::Equal,
                                Opcode::NotEqual => BinaryOperator::NotEqual,
                                Opcode::LessThan => BinaryOperator::Less,
                                Opcode::LessThanEqual => BinaryOperator::LessEqual,
                                _ => unreachable!(),
                            };
                            let left = self.load_src_register(&dst)?;
                            let res = self.evaluate_expr(Expression::Binary {
                                op: bin_op,
                                left,
                                right: src,
                            });

                            // Comparison opcodes appears to compare the src and dst, and then
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
                        Opcode::Mod => {
                            let dst_val = self.load_src_register(&dst)?;
                            self.evaluate_expr(Expression::Binary {
                                op: BinaryOperator::Modulo,
                                left: dst_val,
                                right: src,
                            })
                        }
                        Opcode::FloatToInt => self.evaluate_expr(Expression::As {
                            kind: crate::ScalarKind::Sint,
                            expr: src,
                            convert: Some(4),
                        }),
                        Opcode::IntToFloat => self.evaluate_expr(Expression::As {
                            kind: crate::ScalarKind::Float,
                            expr: src,
                            convert: Some(4),
                        }),
                        Opcode::CrossProduct => {
                            let src_val = self.load_src_register_with_padding(src_reg, false)?;
                            let dst_val = self.load_src_register_with_padding(&dst, false)?;
                            let res = self.evaluate_expr(Expression::Math {
                                fun: MathFunction::Cross,
                                arg: dst_val,
                                arg1: Some(src_val),
                                arg2: None,
                                arg3: None,
                            });
                            self.pad_result(res, src_reg.is_scalar())
                        }
                        Opcode::Fract => self.evaluate_expr(Expression::Math {
                            fun: MathFunction::Fract,
                            arg: src,
                            arg1: None,
                            arg2: None,
                            arg3: None,
                        }),
                        _ => {
                            panic!("Unimplemented opcode {opcode:?}");
                        }
                    };
                    self.emit_dest_store(evaluated, &dst)?;
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

                    let sample_result = self.sample_texture(
                        sample_wrapper_func,
                        normalized_coord,
                        image,
                        sampler,
                        zeroed_out_of_range_expr,
                    );

                    self.emit_dest_store(sample_result, dst)?;
                }
                Operation::LoadFloat { dst, val } => {
                    let val_expr = self
                        .func
                        .expressions
                        .append(Expression::Literal(Literal::F32(*val)), Span::UNDEFINED);
                    let const_vec = self.evaluate_expr(Expression::Splat {
                        size: naga::VectorSize::Quad,
                        value: val_expr,
                    });
                    self.emit_dest_store(const_vec, dst)?;
                }
                Operation::LoadInt { dst, val } => {
                    let val_expr = self
                        .func
                        .expressions
                        .append(Expression::Literal(Literal::I32(*val)), Span::UNDEFINED);
                    let const_vec = self.evaluate_expr(Expression::Splat {
                        size: naga::VectorSize::Quad,
                        value: val_expr,
                    });
                    self.emit_dest_store(const_vec, dst)?;
                }
                Operation::If { src } => {
                    let expr_zero = match src.kind {
                        PixelBenderRegKind::Float => self.zerof32,
                        PixelBenderRegKind::Int => self.zeroi32,
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
                Operation::Select {
                    src1,
                    src2,
                    dst,
                    condition,
                } => {
                    let src1_expr = self.load_src_register(src1)?;
                    let src2_expr = self.load_src_register(src2)?;

                    let expr_zero: Handle<Expression> = match condition.kind {
                        PixelBenderRegKind::Float => self.zerof32,
                        PixelBenderRegKind::Int => self.zeroi32,
                    };
                    if condition.channels.len() != 1 {
                        panic!("'Select' condition must be a scalar: {condition:?}");
                    }

                    // FIXME - `load_src_register` always gives us a vec4 - ideally, we would
                    // have a flag to avoid this pointless splat-and-extract.
                    let cond_expr = self.load_src_register(condition)?;
                    let first_component = self.evaluate_expr(Expression::AccessIndex {
                        base: cond_expr,
                        index: 0,
                    });

                    let is_true = self.evaluate_expr(Expression::Binary {
                        op: BinaryOperator::NotEqual,
                        left: first_component,
                        right: expr_zero,
                    });

                    let select_expr = self.evaluate_expr(Expression::Select {
                        condition: is_true,
                        accept: src1_expr,
                        reject: src2_expr,
                    });

                    self.emit_dest_store(select_expr, dst)?;
                }
                Operation::Nop => {}
            }
        }
        Ok(())
    }

    /// Gets a pointer to the given register - this does *not* perform a load, so it can
    /// be used with both `Expression::Load` and `Statement::Store`
    fn register_pointer(&mut self, reg: &PixelBenderReg) -> Result<Handle<Expression>> {
        let base_index = reg.index as usize;

        if matches!(
            reg.channels[0],
            PixelBenderRegChannel::M2x2 | PixelBenderRegChannel::M3x3 | PixelBenderRegChannel::M4x4
        ) {
            panic!("register_pointer cannot be used with matrix channel {reg:?}")
        }

        let (ty, registers, register_kind_name) = match reg.kind {
            PixelBenderRegKind::Float => (self.vec4f, &mut self.float_registers, "float"),
            PixelBenderRegKind::Int => (self.vec4i, &mut self.int_registers, "int"),
        };

        if base_index >= registers.len() {
            registers.resize(base_index + 1, None);
        }

        if registers[base_index].is_none() {
            let local = self.func.local_variables.append(
                LocalVariable {
                    name: Some(format!("local_{register_kind_name}_reg_{base_index}")),
                    ty,
                    init: None,
                },
                Span::UNDEFINED,
            );

            let expr = self
                .func
                .expressions
                .append(Expression::LocalVariable(local), Span::UNDEFINED);
            registers[base_index] = Some(expr);
        }

        Ok(registers[base_index].unwrap())
    }

    fn load_src_register(&mut self, reg: &PixelBenderReg) -> Result<Handle<Expression>> {
        self.load_src_register_with_padding(reg, true)
    }

    /// Loads a vec4f/vec4i from the given register. Note that all registers are 4-component
    /// vectors - if the `PixelBenderReg` requests fewer components then that, then the extra
    /// components will be meaningless. This greatly simplifies the code, since we don't need
    /// to track whether or not we have a scalar or a vector everywhere.
    fn load_src_register_with_padding(
        &mut self,
        reg: &PixelBenderReg,
        padding: bool,
    ) -> Result<Handle<Expression>> {
        if matches!(
            reg.channels.as_slice(),
            [PixelBenderRegChannel::M2x2]
                | [PixelBenderRegChannel::M3x3]
                | [PixelBenderRegChannel::M4x4]
        ) {
            assert_eq!(
                reg.kind,
                PixelBenderRegKind::Float,
                "Unexpected matrix element type"
            );

            let get_vec_ptr = |this: &mut Self, index| {
                this.register_pointer(&PixelBenderReg {
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    index: reg.index + index,
                    kind: PixelBenderRegKind::Float,
                })
            };
            return self.load_reg_as_matrix(reg.channels[0], get_vec_ptr);
        }

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
                _ => panic!("Unexpected source register channel: {c:?}"),
            })
            .collect::<Vec<_>>();

        let size = if padding {
            VectorSize::Quad
        } else {
            match reg.channels.len() {
                1 => {
                    return Ok(self.evaluate_expr(Expression::AccessIndex {
                        base: reg_value,
                        index: swizzle_components[0] as u32,
                    }))
                }
                2 => VectorSize::Bi,
                3 => VectorSize::Tri,
                4 => VectorSize::Quad,
                _ => unreachable!(),
            }
        };

        if swizzle_components.len() < 4 {
            // Pad with W - these components will be ignored, since whatever uses
            // the result will only use the components corresponding to 'reg.channels'
            swizzle_components.resize(4_usize, SwizzleComponent::W);
        }

        Ok(self.evaluate_expr(Expression::Swizzle {
            size,
            vector: reg_value,
            pattern: swizzle_components.try_into().unwrap(),
        }))
    }

    /// Creates a `Statement::Emit` covering `expr`
    fn evaluate_expr(&mut self, expr: Expression) -> Handle<Expression> {
        evaluate_expr(&mut self.func, &mut self.blocks, expr)
    }

    /// Normally, we pad all loads (including scalar loads) to a vec4, and operate component-wise
    /// on them. This removes the need to check for scalar vs vector everywhere.
    ///
    /// However, some operations
    /// will give a different result if we pad out to a vec4 (e.g. Sqrt, Equal, DotProduct).
    /// For these operations, we work with the original un-padded register load (possibly a scalar).
    /// To simplify the rest of the code, we then pad the *result* to a vec4, which allows the dest
    /// writing code to operate on a vector component-wise, and not worry about scalar vs vector.
    /// (the dest mask ensures that the padding is not written).
    ///
    /// This function pads out a result to a vec4 if it was a scalar. We leave other vector
    /// types (e.g. vec3) unmodified, since they can still be used with AccessIndex
    fn pad_result(&mut self, result: Handle<Expression>, is_scalar: bool) -> Handle<Expression> {
        if is_scalar {
            self.evaluate_expr(Expression::Splat {
                size: VectorSize::Quad,
                value: result,
            })
        } else {
            result
        }
    }

    // Emits a store of `expr` to the destination register, taking into account the store mask.
    fn emit_dest_store(&mut self, expr: Handle<Expression>, dst: &PixelBenderReg) -> Result<()> {
        if matches!(
            dst.channels.as_slice(),
            [PixelBenderRegChannel::M2x2]
                | [PixelBenderRegChannel::M3x3]
                | [PixelBenderRegChannel::M4x4]
        ) {
            // If we're writing to a 2x2 matrix, load the individual values from the matrix,
            // and construct a vec4f containing all of them (a 2x2 matrix is stored as a single vec4f)
            if let PixelBenderRegChannel::M2x2 = dst.channels[0] {
                let col0 = self.evaluate_expr(Expression::AccessIndex {
                    base: expr,
                    index: 0,
                });
                let val0 = self.evaluate_expr(Expression::AccessIndex {
                    base: col0,
                    index: 0,
                });
                let val1 = self.evaluate_expr(Expression::AccessIndex {
                    base: col0,
                    index: 1,
                });

                let col1 = self.evaluate_expr(Expression::AccessIndex {
                    base: expr,
                    index: 1,
                });
                let val2 = self.evaluate_expr(Expression::AccessIndex {
                    base: col1,
                    index: 0,
                });
                let val3 = self.evaluate_expr(Expression::AccessIndex {
                    base: col1,
                    index: 1,
                });

                let combined_vec = self.evaluate_expr(Expression::Compose {
                    ty: self.vec4f,
                    components: vec![val0, val1, val2, val3],
                });
                let dst_register = self.register_pointer(&PixelBenderReg {
                    channels: PixelBenderRegChannel::RGBA.to_vec(),
                    index: dst.index,
                    kind: PixelBenderRegKind::Float,
                })?;
                self.push_statement(Statement::Store {
                    pointer: dst_register,
                    value: combined_vec,
                })
            } else {
                // If we're writing to a 3x3 or 4x4 matrix, load each column from the matrix,
                // and store it in a float registers. Matrices are stored in column-major order.
                let (num_cols, components) = match dst.channels[0] {
                    PixelBenderRegChannel::M3x3 => (3, PixelBenderRegChannel::RGB.to_vec()),
                    PixelBenderRegChannel::M4x4 => (4, PixelBenderRegChannel::RGBA.to_vec()),
                    _ => unreachable!(),
                };
                for i in 0..num_cols {
                    let col = self.evaluate_expr(Expression::AccessIndex {
                        base: expr,
                        index: i,
                    });
                    self.emit_dest_store(
                        col,
                        &PixelBenderReg {
                            channels: components.clone(),
                            index: dst.index + i,
                            kind: PixelBenderRegKind::Float,
                        },
                    )?;
                }
            }
            return Ok(());
        }

        let dst_register = self.register_pointer(dst).unwrap();

        for (dst_channel, src_channel) in
            dst.channels.iter().zip(PixelBenderRegChannel::RGBA.iter())
        {
            if matches!(
                dst_channel,
                PixelBenderRegChannel::M2x2
                    | PixelBenderRegChannel::M3x3
                    | PixelBenderRegChannel::M4x4
            ) {
                panic!("Unexpected to matrix channel for dst {dst:?}");
            }
            // Write each channel of the source to the channel specified by the destination mask
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
        Ok(())
    }

    /// Pushes a statement, taking into account our current 'if' block.
    /// Use this instead of `self.func.body.push`
    fn push_statement(&mut self, stmt: Statement) {
        push_statement(&mut self.blocks, stmt)
    }

    // Loads a Matrix with a size determined by `channel`. Each column of the matrix
    // is loaded via the `get_vec_ptr` callback.
    fn load_reg_as_matrix(
        &mut self,
        channel: PixelBenderRegChannel,
        mut get_vec_ptr: impl FnMut(&mut Self, u32) -> Result<Handle<Expression>>,
    ) -> Result<Handle<Expression>> {
        let vec0_ptr = get_vec_ptr(self, 0)?;
        let vec0_load = self.evaluate_expr(Expression::Load { pointer: vec0_ptr });

        match channel {
            // FIXME - add tests for this case
            PixelBenderRegChannel::M2x2 => {
                // A 2x2 matrix fits exactly into our single vec4f
                // Only the first two components of `pattern` matter
                let col0 = self.evaluate_expr(Expression::Swizzle {
                    size: VectorSize::Bi,
                    vector: vec0_load,
                    pattern: [
                        SwizzleComponent::X,
                        SwizzleComponent::Y,
                        SwizzleComponent::W,
                        SwizzleComponent::W,
                    ],
                });

                // Only the first two components of `pattern` matter (load the Z and W components into the second row)
                let col1 = self.evaluate_expr(Expression::Swizzle {
                    size: VectorSize::Bi,
                    vector: vec0_load,
                    pattern: [
                        SwizzleComponent::Z,
                        SwizzleComponent::W,
                        SwizzleComponent::W,
                        SwizzleComponent::W,
                    ],
                });

                Ok(self.evaluate_expr(Expression::Compose {
                    ty: self.mat2x2f,
                    components: vec![col0, col1],
                }))
            }
            PixelBenderRegChannel::M3x3 | PixelBenderRegChannel::M4x4 => {
                let vec1_ptr = get_vec_ptr(self, 1)?;
                let mut col1 = self.evaluate_expr(Expression::Load { pointer: vec1_ptr });

                let vec2_ptr = get_vec_ptr(self, 2)?;
                let mut col2 = self.evaluate_expr(Expression::Load { pointer: vec2_ptr });

                match channel {
                    PixelBenderRegChannel::M3x3 => {
                        let col0 = self.evaluate_expr(Expression::Swizzle {
                            size: VectorSize::Tri,
                            vector: vec0_load,
                            pattern: [
                                SwizzleComponent::X,
                                SwizzleComponent::Y,
                                SwizzleComponent::Z,
                                SwizzleComponent::W,
                            ],
                        });

                        col1 = self.evaluate_expr(Expression::Swizzle {
                            size: VectorSize::Tri,
                            vector: col1,
                            pattern: [
                                SwizzleComponent::X,
                                SwizzleComponent::Y,
                                SwizzleComponent::Z,
                                SwizzleComponent::W,
                            ],
                        });

                        col2 = self.evaluate_expr(Expression::Swizzle {
                            size: VectorSize::Tri,
                            vector: col2,
                            pattern: [
                                SwizzleComponent::X,
                                SwizzleComponent::Y,
                                SwizzleComponent::Z,
                                SwizzleComponent::W,
                            ],
                        });

                        Ok(self.evaluate_expr(Expression::Compose {
                            ty: self.mat3x3f,
                            components: vec![col0, col1, col2],
                        }))
                    }
                    // FIXME - add tests for this case
                    PixelBenderRegChannel::M4x4 => {
                        let vec3_ptr = get_vec_ptr(self, 3)?;
                        let col3 = self.evaluate_expr(Expression::Load { pointer: vec3_ptr });

                        Ok(self.evaluate_expr(Expression::Compose {
                            ty: self.mat4x4f,
                            components: vec![vec0_load, col1, col2, col3],
                        }))
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
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

fn evaluate_expr(
    func: &mut Function,
    blocks: &mut [BlockStackEntry],
    expr: Expression,
) -> Handle<Expression> {
    let prev_len = func.expressions.len();
    let expr = func.expressions.append(expr, Span::UNDEFINED);
    let range = func.expressions.range_from(prev_len);
    push_statement(blocks, Statement::Emit(range));
    expr
}

fn push_statement(blocks: &mut [BlockStackEntry], stmt: Statement) {
    let block = match blocks.last_mut().unwrap() {
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
