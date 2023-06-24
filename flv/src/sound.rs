#[repr(u8)]
pub enum SoundFormat {
    LinearPCMPlatformEndian = 0,
    Adpcm = 1,
    MP3 = 2,
    LinearPCMLittleEndian = 3,
    Nellymoser16kHz = 4,
    Nellymoser8kHz = 5,
    Nellymoser = 6,
    G711ALawPCM = 7,
    G711MuLawPCM = 8,
    Aac = 10,
    Speex = 11,
    MP38kHz = 14,
    DeviceSpecific = 15,
}

#[repr(u8)]
pub enum SoundRate {
    R5_500 = 0,
    R11_000 = 1,
    R22_000 = 2,
    R44_000 = 3,
}

#[repr(u8)]
pub enum SoundSize {
    Bits8 = 0,
    Bits16 = 1,
}

#[repr(u8)]
pub enum SoundType {
    Mono = 0,
    Stereo = 1,
}
