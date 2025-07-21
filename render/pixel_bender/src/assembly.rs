use std::{collections::HashMap, fmt::Display, io::Write, sync::LazyLock};

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};
use num_traits::Num;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use thiserror::Error;

use crate::{
    Opcode, PixelBenderParamQualifier, PixelBenderRegChannel, PixelBenderRegKind,
    PixelBenderTypeOpcode,
};

#[derive(Parser)]
#[grammar = "assembly_grammar.pest"]
struct PbasmParser;

#[derive(Debug)]
pub struct PbasmErrorLocation(usize, usize);

impl PbasmErrorLocation {
    fn from(line_col: (usize, usize)) -> Self {
        Self(line_col.0, line_col.1)
    }
}

impl Display for PbasmErrorLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.0, self.1)
    }
}

#[derive(Error, Debug)]
pub enum PbasmError {
    #[error("Parsing failed: {0}")]
    ParsingError(Box<pest::error::Error<Rule>>),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}: Unknown opcode: {1}")]
    UnknownOpcode(PbasmErrorLocation, String),

    #[error("{0}: Unknown type: {1}")]
    UnknownType(PbasmErrorLocation, String),

    #[error("{0}: Too few arguments")]
    TooFewArguments(PbasmErrorLocation),

    #[error("{0}: Too many arguments")]
    TooManyArguments(PbasmErrorLocation),

    #[error("{0}: Unexpected argument type ({1:?}), expected {2:?}")]
    WrongArgument(PbasmErrorLocation, Rule, Rule),

    #[error("{0}: Error parsing integer {1}")]
    ErrorParsingInt(PbasmErrorLocation, String),

    #[error("{0}: Error parsing float {1}")]
    ErrorParsingFloat(PbasmErrorLocation, String),

    #[error("{0}: Error parsing string {1}: {2}")]
    ErrorParsingString(PbasmErrorLocation, String, Box<serde_json::Error>),

    #[error("{0}: String too long, {1}>{2}")]
    StringTooLong(PbasmErrorLocation, usize, usize),

    #[error("{0}: Bad register index: {1}")]
    BadRegisterIndex(PbasmErrorLocation, String),

    #[error("{0}: Bad register channels")]
    BadRegisterChannels(PbasmErrorLocation),

    #[error("{0}: Matrix register not allowed")]
    MatrixRegisterNotAllowed(PbasmErrorLocation),

    #[error("{0}: Destination matrix register has to match the source")]
    BadMatrixDstRegister(PbasmErrorLocation),

    #[error("{0}: Swizzle not allowed in destination registers")]
    SwizzleNotAllowed(PbasmErrorLocation),
}

type PbasmResult<T> = Result<T, PbasmError>;

enum RegisterType {
    Parameter,
    Destination { extra_size: RegisterSize },
    Source,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RegisterSize {
    None,
    Standard(u8),
    Matrix(u8),
}

impl RegisterSize {
    fn valid(self) -> bool {
        match self {
            RegisterSize::Standard(s) => 1 <= s && s <= 4,
            RegisterSize::Matrix(s) => 2 <= s && s <= 4,
            RegisterSize::None => true,
        }
    }

    fn is_matrix(self) -> bool {
        matches!(self, RegisterSize::Matrix(_))
    }
}

static COMMON_OPCODE_TABLE: LazyLock<HashMap<String, Opcode>> = LazyLock::new(|| {
    let mut table = HashMap::new();
    table.insert("abs".to_string(), Opcode::Abs);
    table.insert("acos".to_string(), Opcode::Acos);
    table.insert("add".to_string(), Opcode::Add);
    table.insert("all.b".to_string(), Opcode::BoolAll);
    table.insert("and".to_string(), Opcode::LogicalAnd);
    table.insert("any.b".to_string(), Opcode::BoolAny);
    table.insert("asin".to_string(), Opcode::Asin);
    table.insert("atan".to_string(), Opcode::Atan);
    table.insert("atan2".to_string(), Opcode::Atan2);
    table.insert("b2f".to_string(), Opcode::BoolToFloat);
    table.insert("b2i".to_string(), Opcode::BoolToInt);
    table.insert("ceil".to_string(), Opcode::Ceil);
    table.insert("cos".to_string(), Opcode::Cos);
    table.insert("cross".to_string(), Opcode::CrossProduct);
    table.insert("dist".to_string(), Opcode::Distance);
    table.insert("div".to_string(), Opcode::Div);
    table.insert("dot".to_string(), Opcode::DotProduct);
    table.insert("eq.v".to_string(), Opcode::VectorEqual);
    table.insert("eq".to_string(), Opcode::Equal);
    table.insert("exp".to_string(), Opcode::Exp);
    table.insert("exp2".to_string(), Opcode::Exp2);
    table.insert("f2b".to_string(), Opcode::FloatToBool);
    table.insert("f2i".to_string(), Opcode::FloatToInt);
    table.insert("floor".to_string(), Opcode::Floor);
    table.insert("fract".to_string(), Opcode::Fract);
    table.insert("i2b".to_string(), Opcode::IntToBool);
    table.insert("i2f".to_string(), Opcode::IntToFloat);
    table.insert("le".to_string(), Opcode::LessThanEqual);
    table.insert("len".to_string(), Opcode::Length);
    table.insert("log".to_string(), Opcode::Log);
    table.insert("log2".to_string(), Opcode::Log2);
    table.insert("lt".to_string(), Opcode::LessThan);
    table.insert("max".to_string(), Opcode::Max);
    table.insert("min".to_string(), Opcode::Min);
    table.insert("mod".to_string(), Opcode::Mod);
    table.insert("mov".to_string(), Opcode::Mov);
    table.insert("mul.mm".to_string(), Opcode::MatMatMul);
    table.insert("mul.mv".to_string(), Opcode::MatVecMul);
    table.insert("mul.vm".to_string(), Opcode::VecMatMul);
    table.insert("mul".to_string(), Opcode::Mul);
    table.insert("neq.v".to_string(), Opcode::VectorNotEqual);
    table.insert("neq".to_string(), Opcode::NotEqual);
    table.insert("norm".to_string(), Opcode::Normalize);
    table.insert("not".to_string(), Opcode::LogicalNot);
    table.insert("or".to_string(), Opcode::LogicalOr);
    table.insert("pow".to_string(), Opcode::Pow);
    table.insert("rcp".to_string(), Opcode::Rcp);
    table.insert("rsqrt".to_string(), Opcode::RSqrt);
    table.insert("sign".to_string(), Opcode::Sign);
    table.insert("sin".to_string(), Opcode::Sin);
    table.insert("sqrt".to_string(), Opcode::Sqrt);
    table.insert("step".to_string(), Opcode::Step);
    table.insert("sub".to_string(), Opcode::Sub);
    table.insert("tan".to_string(), Opcode::Tan);
    table.insert("xor".to_string(), Opcode::LogicalXor);
    table
});

pub struct PixelBenderShaderAssembly<'a> {
    input: &'a str,
    write: &'a mut dyn Write,
}

impl<'a> PixelBenderShaderAssembly<'a> {
    pub fn new(input: &'a str, write: &'a mut dyn Write) -> Self {
        Self { input, write }
    }

    pub fn assemble(mut self) -> PbasmResult<()> {
        let pairs = PbasmParser::parse(Rule::shader, self.input)
            .map_err(|e| PbasmError::ParsingError(Box::new(e)))?;
        self.assemble_shader(pairs)
    }

    fn assemble_shader(&mut self, pairs: Pairs<'_, Rule>) -> PbasmResult<()> {
        for pair in pairs {
            match pair.as_rule() {
                Rule::EOI => break,
                Rule::line => {
                    self.assemble_line(pair.into_inner())?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn assemble_line(&mut self, mut pairs: Pairs<'_, Rule>) -> PbasmResult<()> {
        let Some(pair) = pairs.next().filter(|p| p.as_rule() == Rule::statement) else {
            // Ignore non-statements
            return Ok(());
        };

        self.assemble_statement(pair.into_inner())
    }

    fn assemble_statement(&mut self, mut pairs: Pairs<'_, Rule>) -> PbasmResult<()> {
        let opcode_pair = pairs
            .next()
            .filter(|p| p.as_rule() == Rule::opcode)
            .expect("Statement should start with an opcode");
        let opcode = opcode_pair.as_str();

        let mut arguments = pairs;

        match opcode {
            "version" => {
                let arg = self.next_arg(&opcode_pair, &mut arguments)?;
                let arg = self.argument_as_int(arg)?;
                self.write.write_u8(Opcode::Version as u8)?;
                self.write.write_i32::<LittleEndian>(arg)?;
            }
            "name" => {
                let arg = self.next_arg(&opcode_pair, &mut arguments)?;
                let line_col = arg.line_col();
                let name = self.argument_as_string(arg)?;

                let len = u16::try_from(name.len()).map_err(|_| {
                    PbasmError::StringTooLong(
                        PbasmErrorLocation::from(line_col),
                        name.len(),
                        u16::MAX.into(),
                    )
                })?;

                self.write.write_u8(Opcode::Name as u8)?;
                self.write.write_u16::<LittleEndian>(len)?;
                self.write.write_all(name.as_bytes())?;
            }
            "param.in" | "param.out" => {
                let name = self.next_arg(&opcode_pair, &mut arguments)?;
                let param_type = self.next_arg(&opcode_pair, &mut arguments)?;
                let reg = self.next_arg(&opcode_pair, &mut arguments)?;

                let name = self.argument_as_string(name)?;
                let param_type = self.argument_as_type(param_type)?;
                let (_, reg) = self.argument_as_reg(reg, RegisterType::Parameter, true)?;

                let qualifier = if opcode.ends_with(".in") {
                    PixelBenderParamQualifier::Input
                } else {
                    PixelBenderParamQualifier::Output
                };

                self.write.write_u8(Opcode::PBJParam as u8)?;
                self.write.write_u8(qualifier as u8)?;
                self.write.write_u8(param_type as u8)?;
                self.write.write_all(&reg)?;
                Self::write_string(self.write, &name)?;
            }
            "param.tex" => {
                let name = self.next_arg(&opcode_pair, &mut arguments)?;
                let index = self.next_arg(&opcode_pair, &mut arguments)?;
                let channels = self.next_arg(&opcode_pair, &mut arguments)?;

                let name = self.argument_as_string(name)?;
                let index = self.argument_as_int::<u8>(index)?;
                let channels = self.argument_as_int::<u8>(channels)?;

                self.write.write_u8(Opcode::PBJParamTexture as u8)?;
                self.write.write_u8(index)?;
                self.write.write_u8(channels)?;
                Self::write_string(self.write, &name)?;
            }
            "meta" | "meta2" => {
                let name = self.next_arg(&opcode_pair, &mut arguments)?;
                let value = self.next_arg(&opcode_pair, &mut arguments)?;

                let name = self.argument_as_string(name)?;
                let (value_type, value) = self.argument_as_typed_value(value)?;

                if opcode == "meta" {
                    self.write.write_u8(Opcode::PBJMeta1 as u8)?;
                } else {
                    self.write.write_u8(Opcode::PBJMeta2 as u8)?;
                }
                self.write.write_u8(value_type as u8)?;
                Self::write_string(self.write, &name)?;
                self.write.write_all(&value)?;
            }
            "smpl.n" | "smpl.l" => {
                let reg_dst = self.next_arg(&opcode_pair, &mut arguments)?;
                let reg_src = self.next_arg(&opcode_pair, &mut arguments)?;
                let tf = self.next_arg(&opcode_pair, &mut arguments)?;

                let tf = self.argument_as_int(tf)?;
                let (_, reg_src) = self.argument_as_reg(reg_src, RegisterType::Source, false)?;
                let (_, reg_dst) = self.argument_as_reg(
                    reg_dst,
                    RegisterType::Destination {
                        extra_size: RegisterSize::None,
                    },
                    false,
                )?;

                if opcode.ends_with(".n") {
                    self.write.write_u8(Opcode::SampleNearest as u8)?;
                } else {
                    self.write.write_u8(Opcode::SampleLinear as u8)?;
                }
                self.write.write_all(&reg_dst)?;
                self.write.write_all(&reg_src)?;
                self.write.write_u8(tf)?;
            }
            "ld" => {
                let reg_dst = self.next_arg(&opcode_pair, &mut arguments)?;
                let immediate = self.next_arg(&opcode_pair, &mut arguments)?;

                let (_, reg_dst) = self.argument_as_reg(
                    reg_dst,
                    RegisterType::Destination {
                        extra_size: RegisterSize::None,
                    },
                    false,
                )?;
                let immediate = self.unwrap_pair(immediate, Rule::argument, Some(Rule::literal))?;
                let immediate = self.unwrap_pair(immediate, Rule::literal, None)?;

                self.write.write_u8(Opcode::LoadIntOrFloat as u8)?;
                self.write.write_all(&reg_dst)?;
                match immediate.as_rule() {
                    Rule::literal_int => {
                        let immediate = self.get_literal_int(immediate)?;
                        self.write.write_i32::<LittleEndian>(immediate)?;
                    }
                    Rule::literal_float => {
                        let immediate = self.get_literal_float(immediate)?;
                        self.write.write_f32::<BigEndian>(immediate)?;
                    }
                    _ => unreachable!(),
                }
            }
            "nop" => {
                self.write.write_u8(Opcode::Nop as u8)?;
                for _ in 0..6 {
                    self.write.write_u8(0)?;
                }
            }
            "select" => {
                let reg_dst = self.next_arg(&opcode_pair, &mut arguments)?;
                let reg_cond = self.next_arg(&opcode_pair, &mut arguments)?;
                let reg_src1 = self.next_arg(&opcode_pair, &mut arguments)?;
                let reg_src2 = self.next_arg(&opcode_pair, &mut arguments)?;

                let (_, reg_dst) = self.argument_as_reg(
                    reg_dst,
                    RegisterType::Destination {
                        extra_size: RegisterSize::None,
                    },
                    false,
                )?;
                let (_, reg_cond) = self.argument_as_reg(reg_cond, RegisterType::Source, false)?;
                let (_, reg_src1) = self.argument_as_reg(reg_src1, RegisterType::Source, false)?;
                let (_, reg_src2) = self.argument_as_reg(reg_src2, RegisterType::Source, false)?;

                self.write.write_u8(Opcode::Select as u8)?;
                self.write.write_all(&reg_dst)?;
                self.write.write_all(&reg_cond)?;
                self.write.write_u8(0)?;
                self.write.write_all(&reg_src1)?;
                self.write.write_u8(0)?;
                self.write.write_all(&reg_src2)?;
                self.write.write_u8(0)?;
            }
            ".if" => {
                let reg = self.next_arg(&opcode_pair, &mut arguments)?;
                let (_, reg) = self.argument_as_reg(reg, RegisterType::Source, false)?;

                self.write.write_u8(Opcode::If as u8)?;
                for _ in 0..3 {
                    self.write.write_u8(0)?;
                }
                self.write.write_all(&reg)?;
                self.write.write_u8(0)?;
            }
            ".else" => {
                self.write.write_u8(Opcode::Else as u8)?;
                for _ in 0..7 {
                    self.write.write_u8(0)?;
                }
            }
            ".endif" => {
                self.write.write_u8(Opcode::EndIf as u8)?;
                for _ in 0..7 {
                    self.write.write_u8(0)?;
                }
            }
            _ => {
                let Some(&opcode) = COMMON_OPCODE_TABLE.get(opcode) else {
                    return Err(PbasmError::UnknownOpcode(
                        PbasmErrorLocation::from(opcode_pair.line_col()),
                        opcode.to_owned(),
                    ));
                };

                let reg_dst = self.next_arg(&opcode_pair, &mut arguments)?;
                let reg_src = self.next_arg(&opcode_pair, &mut arguments)?;

                let (extra_size, reg_src) =
                    self.argument_as_reg(reg_src, RegisterType::Source, true)?;
                let (_, reg_dst) =
                    self.argument_as_reg(reg_dst, RegisterType::Destination { extra_size }, true)?;

                self.write.write_u8(opcode as u8)?;
                self.write.write_all(&reg_dst)?;
                self.write.write_all(&reg_src)?;
                self.write.write_u8(0u8)?;
            }
        }

        self.check_no_more_args(&opcode_pair, &mut arguments)?;

        Ok(())
    }

    fn next_arg<'b>(
        &self,
        opcode_pair: &Pair<'b, Rule>,
        arguments: &mut Pairs<'b, Rule>,
    ) -> PbasmResult<Pair<'b, Rule>> {
        arguments.next().ok_or_else(|| {
            PbasmError::TooFewArguments(PbasmErrorLocation::from(opcode_pair.line_col()))
        })
    }

    fn check_no_more_args<'b>(
        &self,
        opcode_pair: &Pair<'b, Rule>,
        arguments: &mut Pairs<'b, Rule>,
    ) -> PbasmResult<()> {
        match arguments.next() {
            Some(_) => Err(PbasmError::TooManyArguments(PbasmErrorLocation::from(
                opcode_pair.line_col(),
            ))),
            None => Ok(()),
        }
    }

    fn argument_as_int<T: Num>(&self, argument: Pair<'_, Rule>) -> PbasmResult<T> {
        let inner = self.unwrap_pair(argument, Rule::argument, Some(Rule::literal))?;
        let inner = self.unwrap_pair(inner, Rule::literal, Some(Rule::literal_int))?;
        self.get_literal_int(inner)
    }

    fn argument_as_string(&self, argument: Pair<'_, Rule>) -> PbasmResult<String> {
        let inner = self.unwrap_pair(argument, Rule::argument, Some(Rule::literal))?;
        let inner = self.unwrap_pair(inner, Rule::literal, Some(Rule::literal_string))?;
        self.get_literal_string(inner)
    }

    fn argument_as_type(&self, argument: Pair<'_, Rule>) -> PbasmResult<PixelBenderTypeOpcode> {
        let inner = self.unwrap_pair(argument, Rule::argument, Some(Rule::r#type))?;

        Ok(match inner.as_str() {
            "string" => PixelBenderTypeOpcode::TString,
            "bool" => PixelBenderTypeOpcode::TBool,
            "bool2" => PixelBenderTypeOpcode::TBool2,
            "bool3" => PixelBenderTypeOpcode::TBool3,
            "bool4" => PixelBenderTypeOpcode::TBool4,
            "int" => PixelBenderTypeOpcode::TInt,
            "int2" => PixelBenderTypeOpcode::TInt2,
            "int3" => PixelBenderTypeOpcode::TInt3,
            "int4" => PixelBenderTypeOpcode::TInt4,
            "float" => PixelBenderTypeOpcode::TFloat,
            "float2" => PixelBenderTypeOpcode::TFloat2,
            "float3" => PixelBenderTypeOpcode::TFloat3,
            "float4" => PixelBenderTypeOpcode::TFloat4,
            "float2x2" => PixelBenderTypeOpcode::TFloat2x2,
            "float3x3" => PixelBenderTypeOpcode::TFloat3x3,
            "float4x4" => PixelBenderTypeOpcode::TFloat4x4,
            _ => {
                return Err(PbasmError::UnknownType(
                    PbasmErrorLocation::from(inner.line_col()),
                    inner.as_str().to_string(),
                ));
            }
        })
    }

    fn argument_as_typed_value(
        &self,
        argument: Pair<'_, Rule>,
    ) -> PbasmResult<(PixelBenderTypeOpcode, Vec<u8>)> {
        let inner = self.unwrap_pair(argument, Rule::argument, Some(Rule::typed_value))?;
        let inner = self.unwrap_pair(inner, Rule::typed_value, None)?;

        // The proper length should be ensured by the grammar.
        let mut value: Vec<u8> = Vec::new();

        let (value_type, is_float) = match inner.as_rule() {
            Rule::typed_value_bool1 => (PixelBenderTypeOpcode::TBool, false),
            Rule::typed_value_bool2 => (PixelBenderTypeOpcode::TBool2, false),
            Rule::typed_value_bool3 => (PixelBenderTypeOpcode::TBool3, false),
            Rule::typed_value_bool4 => (PixelBenderTypeOpcode::TBool4, false),
            Rule::typed_value_int1 => (PixelBenderTypeOpcode::TInt, false),
            Rule::typed_value_int2 => (PixelBenderTypeOpcode::TInt2, false),
            Rule::typed_value_int3 => (PixelBenderTypeOpcode::TInt3, false),
            Rule::typed_value_int4 => (PixelBenderTypeOpcode::TInt4, false),
            Rule::typed_value_float1 => (PixelBenderTypeOpcode::TFloat, true),
            Rule::typed_value_float2 => (PixelBenderTypeOpcode::TFloat2, true),
            Rule::typed_value_float3 => (PixelBenderTypeOpcode::TFloat3, true),
            Rule::typed_value_float4 => (PixelBenderTypeOpcode::TFloat4, true),
            Rule::typed_value_matrix2 => (PixelBenderTypeOpcode::TFloat2x2, true),
            Rule::typed_value_matrix3 => (PixelBenderTypeOpcode::TFloat3x3, true),
            Rule::typed_value_matrix4 => (PixelBenderTypeOpcode::TFloat4x4, true),
            Rule::typed_value_string => (PixelBenderTypeOpcode::TString, false),
            _ => {
                panic!("Unhandled rule: {:?}", inner.as_rule());
            }
        };

        for literal in inner.into_inner() {
            if value_type == PixelBenderTypeOpcode::TString {
                let v = self.get_literal_string(literal)?;
                Self::write_string(&mut value, &v)?;
            } else if is_float {
                let v = self.get_literal_float::<f32>(literal)?;
                value.write_f32::<BigEndian>(v)?;
            } else {
                let v = self.get_literal_int::<i16>(literal)?;
                value.write_i16::<LittleEndian>(v)?;
            }
        }

        Ok((value_type, value))
    }

    fn argument_as_reg(
        &self,
        argument: Pair<'_, Rule>,
        reg_type: RegisterType,
        allow_matrix: bool,
    ) -> PbasmResult<(RegisterSize, Vec<u8>)> {
        let inner = self.unwrap_pair(argument, Rule::argument, Some(Rule::register))?;

        let line_col = inner.line_col();
        let reg_str = inner.as_str();

        let mut parts = inner.into_inner();

        let reg_kind = parts
            .next()
            .filter(|p| p.as_rule() == Rule::reg_kind)
            .expect("register kind");
        let reg_index = parts
            .next()
            .filter(|p| p.as_rule() == Rule::reg_index)
            .expect("register ID");
        let reg_channels = parts
            .next()
            .filter(|p| p.as_rule() == Rule::reg_channels)
            .expect("register channels");

        assert!(parts.next().is_none());

        let kind = match reg_kind.as_str() {
            "i" => PixelBenderRegKind::Int,
            "f" => PixelBenderRegKind::Float,
            _ => unreachable!(),
        };

        let id_base = reg_index.as_str().parse::<i16>().map_err(|_| {
            PbasmError::BadRegisterIndex(PbasmErrorLocation::from(line_col), reg_str.to_string())
        })? as u16;
        let id_offset = if kind != PixelBenderRegKind::Float {
            0x8000u16
        } else {
            0u16
        };
        let id = id_base + id_offset;

        let (channels, size) = match reg_channels.as_str() {
            "m2" => (vec![PixelBenderRegChannel::M2x2], RegisterSize::Matrix(2)),
            "m3" => (vec![PixelBenderRegChannel::M3x3], RegisterSize::Matrix(3)),
            "m4" => (vec![PixelBenderRegChannel::M4x4], RegisterSize::Matrix(4)),
            channels => {
                let channels = channels
                    .chars()
                    .map(|ch| match ch {
                        'r' => PixelBenderRegChannel::R,
                        'g' => PixelBenderRegChannel::G,
                        'b' => PixelBenderRegChannel::B,
                        'a' => PixelBenderRegChannel::A,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();
                let size = RegisterSize::Standard(channels.len() as u8);
                (channels, size)
            }
        };

        if !size.valid() {
            return Err(PbasmError::BadRegisterChannels(PbasmErrorLocation::from(
                line_col,
            )));
        }

        if !allow_matrix && size.is_matrix() {
            return Err(PbasmError::MatrixRegisterNotAllowed(
                PbasmErrorLocation::from(line_col),
            ));
        }

        if let RegisterType::Destination { extra_size } = reg_type {
            if size.is_matrix() && size != extra_size {
                return Err(PbasmError::BadMatrixDstRegister(PbasmErrorLocation::from(
                    line_col,
                )));
            }
        }

        let mut channels_sorted = channels.clone();
        channels_sorted.sort();
        let is_swizzle = channels != channels_sorted;

        let supplement = match reg_type {
            RegisterType::Destination { .. } | RegisterType::Parameter => {
                if is_swizzle {
                    return Err(PbasmError::SwizzleNotAllowed(PbasmErrorLocation::from(
                        line_col,
                    )));
                }

                let mut mask: u8 = 0u8;
                if channels.contains(&PixelBenderRegChannel::R) {
                    mask += 0x8;
                }
                if channels.contains(&PixelBenderRegChannel::G) {
                    mask += 0x4;
                }
                if channels.contains(&PixelBenderRegChannel::B) {
                    mask += 0x2;
                }
                if channels.contains(&PixelBenderRegChannel::A) {
                    mask += 0x1;
                }

                if let RegisterType::Destination { extra_size } = reg_type {
                    let extra = match extra_size {
                        RegisterSize::None => 0,
                        RegisterSize::Standard(size) => size.saturating_sub(1),
                        RegisterSize::Matrix(size) => size.saturating_sub(1) << 2,
                    };
                    (mask << 4) + extra
                } else {
                    if let RegisterSize::Matrix(size) = size {
                        mask += size;
                    };
                    mask
                }
            }
            RegisterType::Source if size.is_matrix() => 0,
            RegisterType::Source => {
                let mut swizzle: u8 = 0u8;
                for i in 0..4 {
                    swizzle <<= 2;
                    if let Some(&channel) = channels.get(i) {
                        swizzle += channel as u8;
                    }
                }
                swizzle
            }
        };

        let mut data = Vec::new();
        data.write_u16::<LittleEndian>(id)?;
        data.write_u8(supplement)?;

        Ok((size, data))
    }

    fn get_literal_int<T: Num>(&self, literal_int: Pair<'_, Rule>) -> PbasmResult<T> {
        let inner = self.unwrap_pair(literal_int, Rule::literal_int, Some(Rule::int))?;
        let int_str = inner.as_str();
        T::from_str_radix(int_str, 10).map_err(|_| {
            PbasmError::ErrorParsingInt(
                PbasmErrorLocation::from(inner.line_col()),
                int_str.to_string(),
            )
        })
    }

    fn get_literal_float<T: Num>(&self, literal_float: Pair<'_, Rule>) -> PbasmResult<T> {
        let inner = self.unwrap_pair(literal_float, Rule::literal_float, Some(Rule::float))?;
        let float_str = inner.as_str();
        T::from_str_radix(float_str, 10).map_err(|_| {
            PbasmError::ErrorParsingInt(
                PbasmErrorLocation::from(inner.line_col()),
                float_str.to_string(),
            )
        })
    }

    fn get_literal_string(&self, literal_string: Pair<'_, Rule>) -> PbasmResult<String> {
        assert_eq!(literal_string.as_rule(), Rule::literal_string);

        let unparsed_string = literal_string.as_str();

        let string: String = serde_json::from_str(unparsed_string).map_err(|e| {
            PbasmError::ErrorParsingString(
                PbasmErrorLocation::from(literal_string.line_col()),
                unparsed_string.to_string(),
                Box::new(e),
            )
        })?;

        Ok(string)
    }

    fn unwrap_pair<'b>(
        &self,
        pair: Pair<'b, Rule>,
        expected_outer: Rule,
        expected_inner: Option<Rule>,
    ) -> PbasmResult<Pair<'b, Rule>> {
        assert_eq!(pair.as_rule(), expected_outer);

        let mut pairs = pair.into_inner();
        let inner = pairs.next();
        assert!(pairs.next().is_none(), "Expected only one inner rule");
        let inner = inner.expect("Expected some inner rule");

        if let Some(expected_inner) = expected_inner {
            if inner.as_rule() != expected_inner {
                return Err(PbasmError::WrongArgument(
                    PbasmErrorLocation::from(inner.line_col()),
                    inner.as_rule(),
                    expected_inner,
                ));
            }
        }

        Ok(inner)
    }

    fn write_string(write: &mut dyn Write, string: &str) -> PbasmResult<()> {
        write.write_all(string.as_bytes())?;
        write.write_u8(0u8)?;
        Ok(())
    }
}
