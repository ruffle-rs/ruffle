use naga::{
    Binding, Expression, FunctionArgument, Handle, Interpolation, LocalVariable, Span,
    StructMember, Type, TypeInner,
};

use crate::{builder::NagaBuilder, Error, ShaderType};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct VaryingRegisters {
    // Whenever we write to a varying register in a vertex shader
    // or read from a varying register in a fragment shader
    // (for the first time), we store the created `Expression` here.
    // See `get_varying_pointer`
    varying_pointers: Vec<Option<VaryingRegister>>,
}

#[derive(Copy, Clone)]
pub struct VaryingRegister {
    expr_local_variable: Handle<Expression>,
    output_struct_index: Option<usize>,
}

impl<'a> NagaBuilder<'a> {
    pub fn get_varying_pointer(&mut self, index: usize) -> Result<Handle<Expression>> {
        if index >= self.varying_registers.varying_pointers.len() {
            self.varying_registers
                .varying_pointers
                .resize(index + 1, None);
        }

        if self.varying_registers.varying_pointers[index].is_none() {
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

                    let output_struct_index =
                        if let TypeInner::Struct { members, .. } = &mut self.return_type.inner {
                            members.push(StructMember {
                                name: Some(format!("varying_{index}")),
                                ty: self.vec4f,
                                binding: Some(Binding::Location {
                                    location: index as u32,
                                    interpolation: Some(naga::Interpolation::Perspective),
                                    sampling: None,
                                    second_blend_source: false,
                                }),
                                offset: 0,
                            });
                            members.len() - 1
                        } else {
                            unreachable!();
                        };

                    self.varying_registers.varying_pointers[index] = Some(VaryingRegister {
                        expr_local_variable: expr,
                        output_struct_index: Some(output_struct_index),
                    })
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
                            second_blend_source: false,
                        }),
                    });
                    let arg_index = self.func.arguments.len() - 1;

                    let expr = self.func.expressions.append(
                        Expression::FunctionArgument(arg_index as u32),
                        Span::UNDEFINED,
                    );
                    self.varying_registers.varying_pointers[index] = Some(VaryingRegister {
                        expr_local_variable: expr,
                        output_struct_index: None,
                    })
                }
            };
        };

        Ok(self.varying_registers.varying_pointers[index]
            .unwrap()
            .expr_local_variable)
    }

    /// Builds the final output struct expression, using the 'main' output (a position or color)
    /// and any varying registers that were written to (if this is a vertex shader)
    pub fn build_output_expr(&mut self, return_ty: Handle<Type>) -> Result<Handle<Expression>> {
        // Load the 'main' output (a position or color) from our temporary location.
        let dest_load = self.evaluate_expr(Expression::Load { pointer: self.dest });
        let mut components = vec![Some(dest_load)];

        // If the vertex shader wrote to any varying registers, we need to
        // return them as well.
        if let ShaderType::Vertex = self.shader_config.shader_type {
            for i in 0..self.varying_registers.varying_pointers.len() {
                if let Some(register) = self.varying_registers.varying_pointers[i] {
                    let component_index = register
                        .output_struct_index
                        .expect("Missing output struct index in vertex shader mode");
                    if component_index >= components.len() {
                        components.resize(component_index + 1, None);
                    }
                    components[component_index] = Some(self.emit_varying_load(i)?);
                }
            }
        }

        let components = components.into_iter().map(|c| c.unwrap()).collect();

        Ok(self.evaluate_expr(Expression::Compose {
            ty: return_ty,
            components,
        }))
    }
}
