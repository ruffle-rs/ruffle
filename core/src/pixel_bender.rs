//! Pixel bender bytecode parsing code.
//! This is heavling based on https://github.com/jamesward/pbjas and https://github.com/HaxeFoundation/format/tree/master/format/pbj

#[cfg(test)]
mod tests;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use num_traits::FromPrimitive;
use std::{
    fmt::{Display, Formatter},
    io::Read,
};

use crate::{
    avm2::{Activation, ArrayObject, ArrayStorage, Error, Value},
    ecma_conversions::f64_to_wrapping_i32,
    string::AvmString,
};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum PixelBenderType {
    TFloat(f32) = 0x1,
    TFloat2(f32, f32) = 0x2,
    TFloat3(f32, f32, f32) = 0x3,
    TFloat4(f32, f32, f32, f32) = 0x4,
    TFloat2x2([f32; 4]) = 0x5,
    TFloat3x3([f32; 9]) = 0x6,
    TFloat4x4([f32; 16]) = 0x7,
    TInt(i16) = 0x8,
    TInt2(i16, i16) = 0x9,
    TInt3(i16, i16, i16) = 0xA,
    TInt4(i16, i16, i16, i16) = 0xB,
    TString(String) = 0xC,
}

impl PixelBenderType {
    pub fn into_avm2_value<'gc>(
        self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        // Flash appears to use a uint/int if the float has no fractional part
        let cv = |f: f32| -> Value<'gc> {
            if f.fract() == 0.0 {
                f64_to_wrapping_i32(f as f64).into()
            } else {
                f.into()
            }
        };
        let vals: Vec<Value<'gc>> = match self {
            PixelBenderType::TString(string) => {
                return Ok(AvmString::new_utf8(activation.context.gc_context, string).into());
            }
            PixelBenderType::TInt(i) => return Ok(i.into()),
            PixelBenderType::TFloat(f) => vec![cv(f)],
            PixelBenderType::TFloat2(f1, f2) => vec![cv(f1), cv(f2)],
            PixelBenderType::TFloat3(f1, f2, f3) => vec![cv(f1), cv(f2), cv(f3)],
            PixelBenderType::TFloat4(f1, f2, f3, f4) => vec![cv(f1), cv(f2), cv(f3), cv(f4)],
            PixelBenderType::TFloat2x2(floats) => floats.iter().map(|f| cv(*f)).collect(),
            PixelBenderType::TFloat3x3(floats) => floats.iter().map(|f| cv(*f)).collect(),
            PixelBenderType::TFloat4x4(floats) => floats.iter().map(|f| cv(*f)).collect(),
            PixelBenderType::TInt2(i1, i2) => vec![i1.into(), i2.into()],
            PixelBenderType::TInt3(i1, i2, i3) => vec![i1.into(), i2.into(), i3.into()],
            PixelBenderType::TInt4(i1, i2, i3, i4) => {
                vec![i1.into(), i2.into(), i3.into(), i4.into()]
            }
        };
        let storage = ArrayStorage::from_args(&vals);
        Ok(ArrayObject::from_storage(activation, storage)?.into())
    }
}

// FIXME - come up with a way to reduce duplication here
#[derive(num_derive::FromPrimitive, Debug, PartialEq)]
pub enum PixelBenderTypeOpcode {
    TFloat = 0x1,
    TFloat2 = 0x2,
    TFloat3 = 0x3,
    TFloat4 = 0x4,
    TFloat2x2 = 0x5,
    TFloat3x3 = 0x6,
    TFloat4x4 = 0x7,
    TInt = 0x8,
    TInt2 = 0x9,
    TInt3 = 0xA,
    TInt4 = 0xB,
    TString = 0xC,
}

#[derive(num_derive::FromPrimitive, Debug, PartialEq)]
pub enum PixelBenderParamQualifier {
    Input = 1,
    Output = 2,
}

impl Display for PixelBenderTypeOpcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PixelBenderTypeOpcode::TFloat => "float",
                PixelBenderTypeOpcode::TFloat2 => "float2",
                PixelBenderTypeOpcode::TFloat3 => "float3",
                PixelBenderTypeOpcode::TFloat4 => "float4",
                PixelBenderTypeOpcode::TFloat2x2 => "matrix2x2",
                PixelBenderTypeOpcode::TFloat3x3 => "matrix3x3",
                PixelBenderTypeOpcode::TFloat4x4 => "matrix4x4",
                PixelBenderTypeOpcode::TInt => "int",
                PixelBenderTypeOpcode::TInt2 => "int2",
                PixelBenderTypeOpcode::TInt3 => "int3",
                PixelBenderTypeOpcode::TInt4 => "int4",
                PixelBenderTypeOpcode::TString => "string",
            }
        )
    }
}

#[derive(num_derive::FromPrimitive, Debug, PartialEq)]
pub enum Opcode {
    Nop = 0x0,
    Add = 0x1,
    Sub = 0x2,
    Mul = 0x3,
    Rcp = 0x4,
    Div = 0x5,
    Atan2 = 0x6,
    Pow = 0x7,
    Mod = 0x8,
    Min = 0x9,
    Max = 0xA,
    Step = 0xB,
    Sin = 0xC,
    Cos = 0xD,
    Tan = 0xE,
    Asin = 0xF,
    Acos = 0x10,
    Atan = 0x11,
    Exp = 0x12,
    Exp2 = 0x13,
    Log = 0x14,
    Log2 = 0x15,
    Sqrt = 0x16,
    RSqrt = 0x17,
    Abs = 0x18,
    Sign = 0x19,
    Floor = 0x1A,
    Ceil = 0x1B,
    Fract = 0x1C,
    Mov = 0x1D,
    FloatToInt = 0x1E,
    IntToFloat = 0x1F,
    MatMatMul = 0x20,
    VecMatMul = 0x21,
    MatVecMul = 0x22,
    Normalize = 0x23,
    Length = 0x24,
    Distance = 0x25,
    DotProduct = 0x26,
    CrossProduct = 0x27,
    Equal = 0x28,
    NotEqual = 0x29,
    LessThan = 0x2A,
    LessThanEqual = 0x2B,
    LogicalNot = 0x2C,
    LogicalAnd = 0x2D,
    LogicalOr = 0x2E,
    LogicalXor = 0x2F,
    SampleNearest = 0x30,
    SampleLinear = 0x31,
    LoadIntOrFloat = 0x32,
    Loop = 0x33,
    If = 0x34,
    Else = 0x35,
    EndIf = 0x36,
    FloatToBool = 0x37,
    BoolToFloat = 0x38,
    IntToBool = 0x39,
    BoolToInt = 0x3A,
    VectorEqual = 0x3B,
    VectorNotEqual = 0x3C,
    BoolAny = 0x3D,
    BoolAll = 0x3E,
    PBJMeta1 = 0xA0,
    PBJParam = 0xA1,
    PBJMeta2 = 0xA2,
    PBJParamTexture = 0xA3,
    Name = 0xA4,
    Version = 0xA5,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
    Nop,
    Normal {
        opcode: Opcode,
        dst: u16,
        mask: u8,
        src: u32,
        other: u8,
    },
    LoadInt {
        dst: u16,
        mask: u8,
        val: i32,
    },
    LoadFloat {
        dst: u16,
        mask: u8,
        val: f32,
    },
    If {
        src: u32,
    },
    Else,
    EndIf,
}

#[derive(Debug, PartialEq)]
pub struct PixelBenderShader {
    pub name: String,
    pub version: i32,
    pub params: Vec<PixelBenderParam>,
    pub metadata: Vec<PixelBenderMetadata>,
    pub operations: Vec<Operation>,
}

#[derive(Debug, PartialEq)]
pub enum PixelBenderParam {
    Normal {
        qualifier: PixelBenderParamQualifier,
        param_type: PixelBenderTypeOpcode,
        reg: u16,
        mask: u8,
        name: String,
        metadata: Vec<PixelBenderMetadata>,
    },
    Texture {
        index: u8,
        channels: u8,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelBenderMetadata {
    pub key: String,
    pub value: PixelBenderType,
}

/// Parses PixelBender bytecode
pub fn parse_shader(mut data: &[u8]) -> PixelBenderShader {
    let mut shader = PixelBenderShader {
        name: String::new(),
        version: 0,
        params: Vec::new(),
        metadata: Vec::new(),
        operations: Vec::new(),
    };
    let data = &mut data;
    let mut metadata = Vec::new();
    while !data.is_empty() {
        read_op(data, &mut shader, &mut metadata).unwrap();
    }
    // Any metadata left in the vec is associated with our final parameter.
    apply_metadata(&mut shader, &mut metadata);
    shader
}

fn read_op<R: Read>(
    data: &mut R,
    shader: &mut PixelBenderShader,
    metadata: &mut Vec<PixelBenderMetadata>,
) -> Result<(), Box<dyn std::error::Error>> {
    let raw = data.read_u8()?;
    let opcode = Opcode::from_u8(raw).expect("Unknown opcode");
    match opcode {
        Opcode::Nop => {
            assert_eq!(data.read_u32::<LittleEndian>()?, 0);
            assert_eq!(data.read_u16::<LittleEndian>()?, 0);
            shader.operations.push(Operation::Nop);
        }
        Opcode::PBJMeta1 | Opcode::PBJMeta2 => {
            let meta_type = data.read_u8()?;
            let meta_key = read_string(data)?;
            let meta_value = read_value(data, PixelBenderTypeOpcode::from_u8(meta_type).unwrap())?;
            metadata.push(PixelBenderMetadata {
                key: meta_key,
                value: meta_value,
            });
        }
        Opcode::PBJParam => {
            let qualifier = data.read_u8()?;
            let param_type = data.read_u8()?;
            let reg = data.read_u16::<LittleEndian>()?;
            let mask = data.read_u8()?;
            let name = read_string(data)?;

            let param_type = PixelBenderTypeOpcode::from_u8(param_type).unwrap_or_else(|| {
                panic!("Unexpected param type {param_type}");
            });
            let qualifier = PixelBenderParamQualifier::from_u8(qualifier)
                .unwrap_or_else(|| panic!("Unexpected param qualifier {qualifier:?}"));
            apply_metadata(shader, metadata);

            shader.params.push(PixelBenderParam::Normal {
                qualifier,
                param_type,
                reg,
                mask,
                name,
                metadata: Vec::new(),
            })
        }
        Opcode::PBJParamTexture => {
            let index = data.read_u8()?;
            let channels = data.read_u8()?;
            let name = read_string(data)?;
            apply_metadata(shader, metadata);

            shader.params.push(PixelBenderParam::Texture {
                index,
                channels,
                name,
            });
        }
        Opcode::Name => {
            let len = data.read_u16::<LittleEndian>()?;
            let mut string_bytes = vec![0; len as usize];
            data.read_exact(&mut string_bytes)?;
            shader.name = String::from_utf8(string_bytes)?;
        }
        Opcode::Version => {
            shader.version = data.read_i32::<LittleEndian>()?;
        }
        Opcode::If => {
            assert_eq!(read_uint24(data)?, 0);
            let src = read_uint24(data)?;
            assert_eq!(data.read_u8()?, 0);
            shader.operations.push(Operation::If { src });
        }
        Opcode::Else => {
            assert_eq!(data.read_u32::<LittleEndian>()?, 0);
            assert_eq!(read_uint24(data)?, 0);
            shader.operations.push(Operation::Else);
        }
        Opcode::EndIf => {
            assert_eq!(data.read_u32::<LittleEndian>()?, 0);
            assert_eq!(read_uint24(data)?, 0);
            shader.operations.push(Operation::EndIf);
        }
        Opcode::LoadIntOrFloat => {
            let dst = data.read_u16::<LittleEndian>()?;
            let mask = data.read_u8()?;
            assert_eq!(mask & 0xF, 0);
            if dst & 0x8000 != 0 {
                let val = data.read_i32::<LittleEndian>()?;
                shader.operations.push(Operation::LoadInt {
                    dst: dst - 0x8000,
                    mask,
                    val,
                })
            } else {
                let val = read_float(data)?;
                shader
                    .operations
                    .push(Operation::LoadFloat { dst, mask, val })
            }
        }
        _ => {
            let dst = data.read_u16::<LittleEndian>()?;
            let mask = data.read_u8()?;
            let src = read_uint24(data)?;
            assert_eq!(data.read_u8()?, 0);
            shader.operations.push(Operation::Normal {
                opcode,
                dst,
                mask,
                src,
                other: 0,
            })
        }
    };
    Ok(())
}

fn read_string<R: Read>(data: &mut R) -> Result<String, Box<dyn std::error::Error>> {
    let mut string = String::new();
    let mut b = data.read_u8()?;
    while b != 0 {
        string.push(b as char);
        b = data.read_u8()?;
    }
    Ok(string)
}

fn read_float<R: Read>(data: &mut R) -> Result<f32, Box<dyn std::error::Error>> {
    Ok(data.read_f32::<BigEndian>()?)
}

fn read_value<R: Read>(
    data: &mut R,
    opcode: PixelBenderTypeOpcode,
) -> Result<PixelBenderType, Box<dyn std::error::Error>> {
    match opcode {
        PixelBenderTypeOpcode::TFloat => Ok(PixelBenderType::TFloat(read_float(data)?)),
        PixelBenderTypeOpcode::TFloat2 => Ok(PixelBenderType::TFloat2(
            read_float(data)?,
            read_float(data)?,
        )),
        PixelBenderTypeOpcode::TFloat3 => Ok(PixelBenderType::TFloat3(
            read_float(data)?,
            read_float(data)?,
            read_float(data)?,
        )),
        PixelBenderTypeOpcode::TFloat4 => Ok(PixelBenderType::TFloat4(
            read_float(data)?,
            read_float(data)?,
            read_float(data)?,
            read_float(data)?,
        )),
        PixelBenderTypeOpcode::TFloat2x2 => Ok(PixelBenderType::TFloat2x2([
            read_float(data)?,
            read_float(data)?,
            read_float(data)?,
            read_float(data)?,
        ])),
        PixelBenderTypeOpcode::TFloat3x3 => {
            let mut floats: [f32; 9] = [0.0; 9];
            for float in &mut floats {
                *float = read_float(data)?;
            }
            Ok(PixelBenderType::TFloat3x3(floats))
        }
        PixelBenderTypeOpcode::TFloat4x4 => {
            let mut floats: [f32; 16] = [0.0; 16];
            for float in &mut floats {
                *float = read_float(data)?;
            }
            Ok(PixelBenderType::TFloat4x4(floats))
        }
        PixelBenderTypeOpcode::TInt => Ok(PixelBenderType::TInt(data.read_i16::<LittleEndian>()?)),
        PixelBenderTypeOpcode::TInt2 => Ok(PixelBenderType::TInt2(
            data.read_i16::<LittleEndian>()?,
            data.read_i16::<LittleEndian>()?,
        )),
        PixelBenderTypeOpcode::TInt3 => Ok(PixelBenderType::TInt3(
            data.read_i16::<LittleEndian>()?,
            data.read_i16::<LittleEndian>()?,
            data.read_i16::<LittleEndian>()?,
        )),
        PixelBenderTypeOpcode::TInt4 => Ok(PixelBenderType::TInt4(
            data.read_i16::<LittleEndian>()?,
            data.read_i16::<LittleEndian>()?,
            data.read_i16::<LittleEndian>()?,
            data.read_i16::<LittleEndian>()?,
        )),
        PixelBenderTypeOpcode::TString => Ok(PixelBenderType::TString(read_string(data)?)),
    }
}

fn read_uint24<R: Read>(data: &mut R) -> Result<u32, Box<dyn std::error::Error>> {
    let mut src = data.read_u16::<LittleEndian>()? as u32;
    src += data.read_u8()? as u32;
    Ok(src)
}

// The opcodes are laid out like this:
//
// ```
// PBJMeta1 (for overall program)
// PBJMeta1 (for overall program)
// PBJParam (param 1)
// ...
// PBJMeta1 (for param 1)
// PBJMeta1 (for param 1)
// ...
// PBJParam (param 2)
// ,,,
// PBJMeta2 (for param 2)
// ```
//
// The metadata associated with parameter is determined by all of the metadata opcodes
// that come after it and before the next parameter opcode. The metadata opcodes
// that come before all params are associated with the overall program.

fn apply_metadata(shader: &mut PixelBenderShader, metadata: &mut Vec<PixelBenderMetadata>) {
    // Reset the accumulated metadata Vec - we will start accumulating metadata for the next param
    let metadata = std::mem::take(metadata);
    if shader.params.is_empty() {
        shader.metadata = metadata;
    } else {
        match shader.params.last_mut().unwrap() {
            PixelBenderParam::Normal { metadata: meta, .. } => {
                *meta = metadata;
            }
            param => {
                if !metadata.is_empty() {
                    panic!("Tried to apply metadata to texture parameter {param:?}")
                }
            }
        }
    }
}
