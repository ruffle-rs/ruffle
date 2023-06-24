use bitflags::bitflags;

bitflags! {
    pub struct TypeFlags: u8 {
        const HAS_AUDIO = 0b1000_0000;
        const HAS_VIDEO = 0b0010_0000;
    }
}

pub struct Header {
    version: u8,
    type_flags: TypeFlags,
    data_offset: u32,
}
