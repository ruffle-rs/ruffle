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

    pub fn apply_stencil_reference(&self, render_pass: &mut wgpu::RenderPass) {
        match self.mode {
            MaskMode::NoMask => {}
            MaskMode::DrawMaskStencil => {
                render_pass.set_stencil_reference(self.count - 1);
            }
            MaskMode::DrawMaskedContent => {
                render_pass.set_stencil_reference(self.count);
            }
            MaskMode::ClearMaskStencil => {
                render_pass.set_stencil_reference(self.count);
            }
        }
    }
}
