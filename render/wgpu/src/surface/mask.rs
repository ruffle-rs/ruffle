use enum_map::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum MaskMode {
    NoMask,
    DrawMaskStencil,
    DrawMaskedContent,
    ClearMaskStencil,
}
