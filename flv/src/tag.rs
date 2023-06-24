use crate::sound::{SoundFormat, SoundRate, SoundSize, SoundType};

#[repr(u8)]
pub enum TagData<'a> {
    Audio {
        format: SoundFormat,
        rate: SoundRate,
        size: SoundSize,
        sound_type: SoundType,
        data: &'a [u8],
    } = 8,
    Video = 9,
    Script = 18,
}

pub struct Tag<'a> {
    timestamp: i32,
    stream_id: u32, //24 bits max
    data: TagData<'a>,
}
