use enum_map::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum MaskMode {
    NoMask,
    DrawMaskStencil,
    DrawMaskedContent,
    ClearMaskStencil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn apply_stencil_reference(self, render_pass: &mut wgpu::RenderPass) {
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

    pub fn push_mask(&mut self) {
        debug_assert!(self.mode == MaskMode::NoMask || self.mode == MaskMode::DrawMaskedContent);
        self.count += 1;
        self.mode = MaskMode::DrawMaskStencil;
    }

    pub fn activate_mask(&mut self) {
        debug_assert!(self.count > 0 && self.mode == MaskMode::DrawMaskStencil);
        self.mode = MaskMode::DrawMaskedContent;
    }

    pub fn deactivate_mask(&mut self) {
        debug_assert!(self.count > 0 && self.mode == MaskMode::DrawMaskedContent);
        self.mode = MaskMode::ClearMaskStencil;
    }

    pub fn pop_mask(&mut self) {
        debug_assert!(self.count > 0 && self.mode == MaskMode::ClearMaskStencil);
        self.count -= 1;
        if self.count == 0 {
            self.mode = MaskMode::NoMask;
        } else {
            self.mode = MaskMode::DrawMaskedContent;
        };
    }
}
