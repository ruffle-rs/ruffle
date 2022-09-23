use std::io::Read;

use naga::{
    ArraySize, BuiltIn, Constant, ConstantInner, EntryPoint, FunctionArgument, FunctionResult,
    GlobalVariable, Interpolation, ScalarValue, ShaderStage, StructMember, SwizzleComponent,
};
use naga::{BinaryOperator, MathFunction};
use naga::{
    Binding, Expression, Function, Handle, LocalVariable, Module, ScalarKind, Span, Statement,
    Type, TypeInner, VectorSize,
};
use num_traits::FromPrimitive;

use crate::{
    types::*, Error, ShaderType, VertexAttributeFormat, ENTRY_POINT, MAX_VERTEX_ATTRIBUTES,
};

const VERTEX_PROGRAM_CONTANTS: u64 = 128;
const FRAGMENT_PROGRAM_CONSTANTS: u64 = 28;

pub type Result<T> = std::result::Result<T, Error>;

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
    // Whenever we read from a varying register in a fragment shader,
    // we create a new argument binding for it.
    argument_expressions: Vec<Option<Handle<Expression>>>,

    varying_pointers: Vec<Option<Handle<Expression>>>,

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
            VertexAttributeFormat::Bytes4 => (VectorSize::Quad, 1, ScalarKind::Uint),
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
                components.push(builder.func.expressions.append(
                    Expression::Constant(builder.module.constants.append(
                        Constant {
                            name: None,
                            specialization: None,
                            inner: ConstantInner::Scalar {
                                width: 4,
                                value: ScalarValue::Float(1.0),
                            },
                        },
                        Span::UNDEFINED,
                    )),
                    Span::UNDEFINED,
                ));
                builder.evaluate_expr(Expression::Compose {
                    ty: builder.vec4f,
                    components,
                })
            }
            VertexAttributeFormat::Float4 => base_expr,
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unsupported conversion from {:?} to float4",
                    self
                )))
            }
        })
    }
}

#[derive(Debug)]
pub struct ShaderConfig<'a> {
    pub shader_type: ShaderType,
    pub vertex_attributes: &'a [Option<VertexAttributeFormat>; 8],
}

impl<'a> NagaBuilder<'a> {
    pub fn process_agal(
        mut agal: &[u8],
        vertex_attributes: &[Option<VertexAttributeFormat>; MAX_VERTEX_ATTRIBUTES],
    ) -> Result<Module> {
        let data = &mut agal;

        let mut header = [0; 7];
        data.read_exact(&mut header)?;

        if header[0..6] != [0xA0, 0x01, 0x00, 0x00, 0x00, 0xA1] {
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
        });

        while !data.is_empty() {
            let mut token = [0; 24];
            data.read_exact(&mut token)?;
            let raw_opcode = u32::from_le_bytes(token[0..4].try_into().unwrap());

            // FIXME - this is a clippy false-positive
            #[allow(clippy::or_fun_call)]
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

        let constant_registers = func.expressions.append(
            Expression::GlobalVariable(constant_registers_global),
            Span::UNDEFINED,
        );

        NagaBuilder {
            module,
            func,
            dest,
            shader_config,
            argument_expressions: vec![],
            varying_pointers: vec![],
            return_type,
            matrix4x4f,
            vec4f,
            constant_registers,
        }
    }

    fn get_vertex_input(&mut self, index: usize) -> Result<Handle<Expression>> {
        if index >= self.argument_expressions.len() {
            self.argument_expressions.resize(index + 1, None);

            // FIXME - this is a clippy false-positive
            #[allow(clippy::or_fun_call)]
            let ty = self.shader_config.vertex_attributes[index]
                .as_ref()
                .ok_or(Error::MissingVertexAttributeData(index))?
                .to_naga_type(&mut self.module);

            self.func.arguments.push(FunctionArgument {
                name: None,
                ty,
                binding: Some(Binding::Location {
                    location: index as u32,
                    interpolation: None,
                    sampling: None,
                }),
            });

            // Arguments map one-to-one to vertex attributes.
            let expr = self
                .func
                .expressions
                .append(Expression::FunctionArgument(index as u32), Span::UNDEFINED);
            self.argument_expressions[index] = Some(expr);
        }
        Ok(self.argument_expressions[index].unwrap())
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
                            name: Some(format!("varying_{}", index)),
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
                            name: Some(format!("varying_{}", index)),
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
                    self.func.arguments.push(FunctionArgument {
                        name: None,
                        ty: self.vec4f,
                        binding: Some(Binding::Location {
                            location: index as u32,
                            interpolation: Some(Interpolation::Perspective),
                            sampling: None,
                        }),
                    });

                    let expr = self
                        .func
                        .expressions
                        .append(Expression::FunctionArgument(index as u32), Span::UNDEFINED);
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

    fn emit_source_field_load(
        &mut self,
        source: &SourceField,
        extend_to_vec4: bool,
    ) -> Result<Handle<Expression>> {
        let (mut base_expr, source_type) = match source.register_type {
            // We can use a function argument directly - we don't need
            // a separate Expression::Load
            RegisterType::Attribute => (
                self.get_vertex_input(source.reg_num as usize)?,
                // FIXME - this is a clippy false-positive
                #[allow(clippy::or_fun_call)]
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

        // Swizzle is 'xyzw', which is a no-op. Just return the base expression.
        if source.swizzle == 0b11100100 {
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

        Ok(self.func.expressions.append(
            Expression::Swizzle {
                size: VectorSize::Quad,
                vector: base_expr,
                pattern: swizzle_components,
            },
            Span::UNDEFINED,
        ))
    }

    fn emit_dest_store(&mut self, dest: &DestField, expr: Handle<Expression>) -> Result<()> {
        let base_expr = match dest.register_type {
            RegisterType::Output => self.dest,
            RegisterType::Varying => self.get_varying_pointer(dest.reg_num as usize)?,
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unimplemented dest reg type: {:?}",
                    dest
                )))
            }
        };

        // Optimization - use a Store instead of writing individual fields
        // when we're writing to the entire output register.
        if dest.write_mask.is_all() {
            let store = Statement::Store {
                pointer: base_expr,
                value: expr,
            };
            self.func.body.push(store, Span::UNDEFINED);
        } else {
            for (i, mask) in [(0, Mask::X), (1, Mask::Y), (2, Mask::Z), (3, Mask::W)] {
                if dest.write_mask.contains(mask) {
                    self.func.body.push(
                        Statement::Store {
                            pointer: self.func.expressions.append(
                                Expression::AccessIndex {
                                    base: base_expr,
                                    index: i,
                                },
                                Span::UNDEFINED,
                            ),
                            value: expr,
                        },
                        Span::UNDEFINED,
                    );
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
        self.func.body.push(Statement::Emit(range), Span::UNDEFINED);
        expr
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
                // On the ActionScript side, the user might have specified something *other* than
                // vec4f. In that case, we need to extend the source to a vec4f if we're writing to
                // a vec4f register.
                // FIXME - do we need to do this extension in other cases?
                let do_extend = matches!(
                    dest.register_type,
                    RegisterType::Output | RegisterType::Varying
                );
                let source = self.emit_source_field_load(source1, do_extend)?;
                self.emit_dest_store(dest, source)?;
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
            _ => {
                return Err(Error::Unimplemented(format!(
                    "Unimplemented opcode: {:?}",
                    opcode
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

        self.func.body.push(
            Statement::Return {
                value: Some(return_expr),
            },
            Span::UNDEFINED,
        );

        let entry_point = EntryPoint {
            name: ENTRY_POINT.to_string(),
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
