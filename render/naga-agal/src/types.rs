use crate::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(num_derive::FromPrimitive, Debug)]
pub enum Opcode {
    Mov = 0x00,
    Add = 0x01,
    Sub = 0x02,
    Mul = 0x03,
    Div = 0x04,
    Rcp = 0x05,
    Min = 0x06,
    Max = 0x07,
    Frc = 0x08,
    Sqt = 0x09,
    Rsq = 0x0a,
    Pow = 0x0b,
    Log = 0x0c,
    Exp = 0x0d,
    Nrm = 0x0e,
    Sin = 0x0f,
    Cos = 0x10,
    Crs = 0x11,
    Dp3 = 0x12,
    Dp4 = 0x13,
    Abs = 0x14,
    Neg = 0x15,
    Sat = 0x16,
    M33 = 0x17,
    M44 = 0x18,
    M34 = 0x19,
    Kil = 0x27,
    Tex = 0x28,
    Sge = 0x29,
    Slt = 0x2a,
    Seq = 0x2c,
    Sne = 0x2d,
    Ddx = 0x1a,
    Ddy = 0x1b,
    Ife = 0x1c,
    Ine = 0x1d,
    Ifg = 0x1e,
    Ifl = 0x1f,
    Els = 0x20,
    Eif = 0x21,
}

#[derive(FromPrimitive, Debug, Clone, PartialEq, Eq)]
pub enum RegisterType {
    Attribute = 0,
    Constant = 1,
    Temporary = 2,
    Output = 3,
    Varying = 4,
    Sampler = 5,
    FragmentRegister = 6,
}

#[derive(Debug, FromPrimitive, Clone)]
pub enum DirectMode {
    Direct = 0,
    Indirect = 1,
}

#[derive(Debug)]
pub struct DestField {
    pub register_type: RegisterType,
    pub write_mask: Mask,
    pub reg_num: u16,
}

impl DestField {
    pub fn parse(val: u32) -> Result<DestField, Error> {
        let reg_num = (val & 0xFFFF) as u16;
        let write_mask = Mask::from_bits(((val >> 16) & 0xF) as u8).unwrap();
        let reg_type = RegisterType::from_u16(((val >> 24) & 0xF) as u16).unwrap();
        Ok(DestField {
            register_type: reg_type,
            write_mask,
            reg_num,
        })
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SourceField {
    pub direct_mode: DirectMode,
    pub index_select: u8,
    pub index_type: RegisterType,
    pub register_type: RegisterType,
    pub swizzle: u8,
    pub indirect_offset: u8,
    pub reg_num: u16,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Mask: u8 {
        const X = 0b0001;
        const Y = 0b0010;
        const Z = 0b0100;
        const W = 0b1000;
    }
}

impl SourceField {
    pub fn parse(val: u64) -> Result<SourceField, Error> {
        // FIXME - check that all the other bits are 0
        let reg_num = (val & 0xFFFF) as u16;
        let indirect_offset = ((val >> 16) & 0xFF) as u8;
        let swizzle = ((val >> 24) & 0xFF) as u8;
        let register_type = RegisterType::from_u16(((val >> 32) & 0xF) as u16).unwrap();
        let index_type = RegisterType::from_u16(((val >> 40) & 0xF) as u16).unwrap();
        let index_select = ((val >> 48) & 0x3) as u8;
        let direct_mode = DirectMode::from_u16(((val >> 63) & 0x1) as u16).unwrap();
        Ok(SourceField {
            direct_mode,
            index_select,
            index_type,
            register_type,
            swizzle,
            indirect_offset,
            reg_num,
        })
    }
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Filter {
    Nearest = 0,
    Linear = 1,
    Anisotropic2x = 2,
    Anisotropic4x = 3,
    Anisotropic8x = 4,
    Anisotropic16x = 5,
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Mipmap {
    Disable = 0,
    Nearest = 1,
    Linear = 2,
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Wrapping {
    Clamp = 0,
    Repeat = 1,
    ClampURepeatV = 2,
    RepeatUClampV = 3,
}

#[derive(FromPrimitive, Debug, Copy, Clone)]
pub enum Dimension {
    TwoD = 0,
    Cube = 1,
}

#[derive(Debug)]
pub struct Special {
    pub ignore_sampler: bool,
}

impl Special {
    pub fn parse(val: u8) -> Result<Special, Error> {
        Ok(Special {
            ignore_sampler: (val & 0x4) != 0,
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct SamplerField {
    pub filter: Filter,
    pub mipmap: Mipmap,
    pub wrapping: Wrapping,
    pub dimension: Dimension,
    pub special: Special,
    /// Texture level-of-detail (LOD) bias
    pub texture_lod_bias: i8,
    pub reg_num: u16,
    pub reg_type: RegisterType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SamplerConfig {
    pub wrapping: Wrapping,
    pub filter: Filter,
    pub mipmap: Mipmap,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        SamplerConfig {
            wrapping: Wrapping::Clamp,
            filter: Filter::Nearest,
            mipmap: Mipmap::Disable,
        }
    }
}

impl SamplerField {
    pub fn parse(val: u64) -> Result<SamplerField, Error> {
        let reg_num = (val & 0xFFFF) as u16;
        let load_bias = ((val >> 16) & 0xFF) as i8;
        let reg_type = RegisterType::from_u64((val >> 32) & 0xF).unwrap();
        let dimension = Dimension::from_u64((val >> 44) & 0xF).unwrap();
        let special = Special::parse(((val >> 48) & 0xF) as u8).unwrap();
        let wrapping = Wrapping::from_u64((val >> 52) & 0xF).unwrap();
        let mipmap = Mipmap::from_u64((val >> 56) & 0xF).unwrap();
        let filter = Filter::from_u64((val >> 60) & 0xF).unwrap();

        Ok(SamplerField {
            filter,
            mipmap,
            wrapping,
            dimension,
            texture_lod_bias: load_bias,
            special,
            reg_num,
            reg_type,
        })
    }
}

#[derive(Debug)]
pub enum Source2 {
    SourceField(SourceField),
    Sampler(SamplerField),
}

impl Source2 {
    pub fn assert_source_field(&self) -> &SourceField {
        match self {
            Source2::SourceField(s) => s,
            _ => panic!("Expected SourceField"),
        }
    }
    pub fn assert_sampler(&self) -> &SamplerField {
        match self {
            Source2::Sampler(s) => s,
            _ => panic!("Expected SamplerField"),
        }
    }
}
