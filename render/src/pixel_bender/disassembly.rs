use std::fmt::{Display, Formatter};

use crate::pixel_bender::{
    Opcode, Operation, PixelBenderMetadata, PixelBenderParam, PixelBenderParamQualifier,
    PixelBenderReg, PixelBenderRegChannel, PixelBenderRegKind, PixelBenderShader, PixelBenderType,
    PixelBenderTypeOpcode,
};

/// Pixel Bender disassembler for debugging purposes.
#[allow(unused)]
pub struct PixelBenderShaderDisassembly<'a>(pub &'a PixelBenderShader);

impl Display for PixelBenderShaderDisassembly<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}",
            self.opcode_to_str(Opcode::Version),
            self.0.version,
        )?;
        writeln!(f, "{} {:?}", self.opcode_to_str(Opcode::Name), self.0.name)?;
        writeln!(f)?;
        self.fmt_metadata(f, &self.0.metadata, "")?;
        if !self.0.metadata.is_empty() {
            writeln!(f)?;
        }
        self.fmt_parameters(f)?;
        writeln!(f)?;
        self.fmt_operations(f)?;
        Ok(())
    }
}

impl PixelBenderShaderDisassembly<'_> {
    fn fmt_metadata(
        &self,
        f: &mut Formatter<'_>,
        metadata: &Vec<PixelBenderMetadata>,
        prefix: &str,
    ) -> std::fmt::Result {
        for meta in metadata {
            write!(f, "{prefix}meta {:?} ", meta.key)?;
            self.fmt_type(f, &meta.value)?;
            writeln!(f)?;
        }
        Ok(())
    }

    fn fmt_parameters(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for param in &self.0.params {
            match param {
                PixelBenderParam::Normal {
                    qualifier,
                    param_type,
                    reg,
                    name,
                    metadata,
                } => {
                    let qualifier = self.qualifier_to_str(*qualifier);
                    let param_type = self.type_to_str(*param_type);
                    write!(
                        f,
                        "{} {qualifier} {name:?} {param_type} ",
                        self.opcode_to_str(Opcode::PBJParam)
                    )?;
                    self.fmt_reg(f, reg)?;
                    if metadata.is_empty() {
                        writeln!(f)?;
                    } else {
                        writeln!(f, " {{")?;
                        self.fmt_metadata(f, metadata, "  ")?;
                        writeln!(f, "}}")?;
                    }
                }
                PixelBenderParam::Texture {
                    index,
                    channels,
                    name,
                } => {
                    writeln!(
                        f,
                        "{} {name:?} {index} {channels}",
                        self.opcode_to_str(Opcode::PBJParamTexture)
                    )?;
                }
            }
        }
        Ok(())
    }

    fn fmt_reg(&self, f: &mut Formatter<'_>, reg: &PixelBenderReg) -> std::fmt::Result {
        match reg.kind {
            PixelBenderRegKind::Float => write!(f, "f"),
            PixelBenderRegKind::Int => write!(f, "i"),
        }?;

        write!(f, "{}", reg.index)?;

        if !reg.channels.is_empty() {
            write!(f, ".")?;
            for ch in &reg.channels {
                f.write_str(self.channel_to_str(ch))?;
            }
        }

        Ok(())
    }

    fn fmt_type(&self, f: &mut Formatter<'_>, type_: &PixelBenderType) -> std::fmt::Result {
        match type_ {
            PixelBenderType::TFloat(a) => write!(f, "float({a})"),
            PixelBenderType::TFloat2(a, b) => write!(f, "float2({a}, {b})"),
            PixelBenderType::TFloat3(a, b, c) => write!(f, "float3({a}, {b}, {c})"),
            PixelBenderType::TFloat4(a, b, c, d) => write!(f, "float4({a}, {b}, {c}, {d})"),
            PixelBenderType::TFloat2x2(a) => write!(f, "float2x2({a:?})"),
            PixelBenderType::TFloat3x3(a) => write!(f, "float3x3({a:?})"),
            PixelBenderType::TFloat4x4(a) => write!(f, "float4x4({a:?})"),
            PixelBenderType::TInt(a) => write!(f, "int({a})"),
            PixelBenderType::TInt2(a, b) => write!(f, "int2({a}, {b})"),
            PixelBenderType::TInt3(a, b, c) => write!(f, "int3({a}, {b}, {c})"),
            PixelBenderType::TInt4(a, b, c, d) => write!(f, "int4({a}, {b}, {c}, {d})"),
            PixelBenderType::TString(a) => write!(f, "string({a:?})"),
            PixelBenderType::TBool(a) => write!(f, "bool({a})"),
            PixelBenderType::TBool2(a, b) => write!(f, "bool2({a}, {b})"),
            PixelBenderType::TBool3(a, b, c) => write!(f, "bool3({a}, {b}, {c})"),
            PixelBenderType::TBool4(a, b, c, d) => write!(f, "bool4({a}, {b}, {c}, {d})"),
        }
    }

    fn fmt_operations(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut prefix = "".to_string();
        for op in &self.0.operations {
            self.fmt_operation(f, op, &mut prefix)?;
        }
        Ok(())
    }

    fn fmt_operation(
        &self,
        f: &mut Formatter<'_>,
        op: &Operation,
        prefix: &mut String,
    ) -> std::fmt::Result {
        match op {
            Operation::Nop => writeln!(f, "{prefix}{}", self.opcode_to_str(Opcode::Nop))?,
            Operation::Normal { opcode, dst, src } => {
                write!(f, "{prefix}{}\t", self.opcode_to_str(*opcode))?;
                self.fmt_reg(f, dst)?;
                write!(f, ", ")?;
                self.fmt_reg(f, src)?;
                writeln!(f)?;
            }
            Operation::LoadInt { dst, val } => {
                write!(f, "{prefix}ld.i\t")?;
                self.fmt_reg(f, dst)?;
                writeln!(f, ", {val}")?;
            }
            Operation::LoadFloat { dst, val } => {
                write!(f, "{prefix}ld.f\t")?;
                self.fmt_reg(f, dst)?;
                writeln!(f, ", {val}")?;
            }
            Operation::If { src } => {
                write!(f, "{prefix}{}\t", self.opcode_to_str(Opcode::If))?;
                self.fmt_reg(f, src)?;
                writeln!(f)?;
                self.prefix_inc(prefix);
            }
            Operation::SampleNearest { dst, src, tf } => {
                write!(f, "{prefix}{}\t", self.opcode_to_str(Opcode::SampleNearest))?;
                self.fmt_reg(f, dst)?;
                write!(f, ", ")?;
                self.fmt_reg(f, src)?;
                writeln!(f, ", {tf}")?;
            }
            Operation::SampleLinear { dst, src, tf } => {
                write!(f, "{prefix}{}\t", self.opcode_to_str(Opcode::SampleLinear))?;
                self.fmt_reg(f, dst)?;
                write!(f, ", ")?;
                self.fmt_reg(f, src)?;
                writeln!(f, ", {tf}")?;
            }
            Operation::Else => {
                self.prefix_dec(prefix);
                writeln!(f, "{prefix}{}", self.opcode_to_str(Opcode::Else))?;
                self.prefix_inc(prefix);
            }
            Operation::EndIf => {
                self.prefix_dec(prefix);
                writeln!(f, "{prefix}{}", self.opcode_to_str(Opcode::EndIf))?;
            }
            Operation::Select {
                src1,
                src2,
                condition,
                dst,
            } => {
                write!(f, "{prefix}{}\t", self.opcode_to_str(Opcode::Select))?;
                self.fmt_reg(f, src1)?;
                write!(f, ", ")?;
                self.fmt_reg(f, src2)?;
                write!(f, ", ")?;
                self.fmt_reg(f, condition)?;
                write!(f, ", ")?;
                self.fmt_reg(f, dst)?;
                writeln!(f)?;
            }
        }
        Ok(())
    }

    fn prefix_inc(&self, prefix: &mut String) {
        prefix.push_str("  ");
    }

    fn prefix_dec(&self, prefix: &mut String) {
        prefix.truncate(prefix.len().saturating_sub(2));
    }

    fn qualifier_to_str(&self, qualifier: PixelBenderParamQualifier) -> &'static str {
        match qualifier {
            PixelBenderParamQualifier::Input => "in",
            PixelBenderParamQualifier::Output => "out",
        }
    }

    fn type_to_str(&self, type_opcode: PixelBenderTypeOpcode) -> &'static str {
        match type_opcode {
            PixelBenderTypeOpcode::TFloat => "float",
            PixelBenderTypeOpcode::TFloat2 => "float2",
            PixelBenderTypeOpcode::TFloat3 => "float3",
            PixelBenderTypeOpcode::TFloat4 => "float4",
            PixelBenderTypeOpcode::TFloat2x2 => "float2x2",
            PixelBenderTypeOpcode::TFloat3x3 => "float3x3",
            PixelBenderTypeOpcode::TFloat4x4 => "float4x4",
            PixelBenderTypeOpcode::TInt => "int",
            PixelBenderTypeOpcode::TInt2 => "int2",
            PixelBenderTypeOpcode::TInt3 => "int3",
            PixelBenderTypeOpcode::TInt4 => "int4",
            PixelBenderTypeOpcode::TString => "string",
            PixelBenderTypeOpcode::TBool => "bool",
            PixelBenderTypeOpcode::TBool2 => "bool2",
            PixelBenderTypeOpcode::TBool3 => "bool3",
            PixelBenderTypeOpcode::TBool4 => "bool4",
        }
    }

    fn channel_to_str(&self, ch: &PixelBenderRegChannel) -> &'static str {
        match ch {
            PixelBenderRegChannel::R => "r",
            PixelBenderRegChannel::G => "g",
            PixelBenderRegChannel::B => "b",
            PixelBenderRegChannel::A => "a",
            PixelBenderRegChannel::M2x2 => "m2",
            PixelBenderRegChannel::M3x3 => "m3",
            PixelBenderRegChannel::M4x4 => "m4",
        }
    }

    fn opcode_to_str(&self, opcode: Opcode) -> &'static str {
        match opcode {
            Opcode::Nop => "nop",
            Opcode::Add => "add",
            Opcode::Sub => "sub",
            Opcode::Mul => "mul",
            Opcode::Rcp => "rcp",
            Opcode::Div => "div",
            Opcode::Atan2 => "atan2",
            Opcode::Pow => "pow",
            Opcode::Mod => "mod",
            Opcode::Min => "min",
            Opcode::Max => "max",
            Opcode::Step => "step",
            Opcode::Sin => "sin",
            Opcode::Cos => "cos",
            Opcode::Tan => "tan",
            Opcode::Asin => "asin",
            Opcode::Acos => "acos",
            Opcode::Atan => "atan",
            Opcode::Exp => "exp",
            Opcode::Exp2 => "exp2",
            Opcode::Log => "log",
            Opcode::Log2 => "log2",
            Opcode::Sqrt => "sqrt",
            Opcode::RSqrt => "rsqrt",
            Opcode::Abs => "abs",
            Opcode::Sign => "sign",
            Opcode::Floor => "floor",
            Opcode::Ceil => "ceil",
            Opcode::Fract => "fract",
            Opcode::Mov => "mov",
            Opcode::FloatToInt => "f2i",
            Opcode::IntToFloat => "i2f",
            Opcode::MatMatMul => "mul.mm",
            Opcode::VecMatMul => "mul.vm",
            Opcode::MatVecMul => "mul.mv",
            Opcode::Normalize => "norm",
            Opcode::Length => "len",
            Opcode::Distance => "dist",
            Opcode::DotProduct => "dot",
            Opcode::CrossProduct => "cross",
            Opcode::Equal => "eq",
            Opcode::NotEqual => "neq",
            Opcode::LessThan => "lt",
            Opcode::LessThanEqual => "lte",
            Opcode::LogicalNot => "not",
            Opcode::LogicalAnd => "and",
            Opcode::LogicalOr => "or",
            Opcode::LogicalXor => "xor",
            Opcode::SampleNearest => "smpl.n",
            Opcode::SampleLinear => "smpl.l",
            Opcode::LoadIntOrFloat => "ld",
            Opcode::Select => "select",
            Opcode::If => ".if",
            Opcode::Else => ".else",
            Opcode::EndIf => ".endif",
            Opcode::FloatToBool => "f2b",
            Opcode::BoolToFloat => "b2f",
            Opcode::IntToBool => "i2b",
            Opcode::BoolToInt => "b2i",
            Opcode::VectorEqual => "eq.v",
            Opcode::VectorNotEqual => "neq.v",
            Opcode::BoolAny => "any.b",
            Opcode::BoolAll => "all.b",
            Opcode::PBJMeta1 => "meta1",
            Opcode::PBJParam => "param",
            Opcode::PBJMeta2 => "meta2",
            Opcode::PBJParamTexture => "param_texture",
            Opcode::Name => "name",
            Opcode::Version => "version",
        }
    }
}
