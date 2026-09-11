use enum_map::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum MaskMode {
    NoMask,
    DrawMaskStencil,
    DrawMaskedContent,
    ClearMaskStencil,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskState {
    pub mode: MaskMode,
    pub count: u32,
}

impl MaskState {
    pub fn empty() -> Self {
        MaskState {
            mode: MaskMode::NoMask,
            count: 0,
        }
    }
}
