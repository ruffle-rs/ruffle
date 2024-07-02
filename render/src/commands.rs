use crate::backend::ShapeHandle;
use crate::bitmap::{BitmapHandle, PixelSnapping};
use crate::matrix::Matrix;
use crate::pixel_bender::PixelBenderShaderHandle;
use crate::transform::Transform;
use swf::{BlendMode, Color};

pub trait CommandHandler {
    fn render_bitmap(
        &mut self,
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
    );
    fn render_stage3d(&mut self, bitmap: BitmapHandle, transform: Transform);
    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform);
    fn draw_rect(&mut self, color: Color, matrix: Matrix);
    fn draw_line(&mut self, color: Color, matrix: Matrix);
    fn draw_line_rect(&mut self, color: Color, matrix: Matrix);
    fn push_mask(&mut self);
    fn activate_mask(&mut self);
    fn deactivate_mask(&mut self);
    fn pop_mask(&mut self);

    fn blend(&mut self, commands: CommandList, blend_mode: RenderBlendMode);
}

/// Holds either a normal BlendMode, or the shader for BlendMode.SHADER.
/// We cannot store the `PixelBenderShaderHandle` directly in `ExtendedBlendMode`,
/// since we need to remember the shader even if the blend mode is changed
/// to something else (so that the shader will still be used if we switch back)
#[derive(Debug, Clone)]
pub enum RenderBlendMode {
    Builtin(BlendMode),
    Shader(PixelBenderShaderHandle),
}

#[derive(Debug, Default, Clone)]
pub struct CommandList {
    pub commands: Vec<Command>,

    /// The number of mask regions in the process of being drawn.
    /// This is used to discard drawing commands of nested maskers, which Flash does not support.
    maskers_in_progress: u32,
}

impl CommandList {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(self, handler: &mut impl CommandHandler) {
        for command in self.commands {
            match command {
                Command::RenderBitmap {
                    bitmap,
                    transform,
                    smoothing,
                    pixel_snapping,
                } => handler.render_bitmap(bitmap, transform, smoothing, pixel_snapping),
                Command::RenderShape { shape, transform } => handler.render_shape(shape, transform),
                Command::RenderStage3D { bitmap, transform } => {
                    handler.render_stage3d(bitmap, transform)
                }
                Command::DrawRect { color, matrix } => handler.draw_rect(color, matrix),
                Command::DrawLine { color, matrix } => handler.draw_line(color, matrix),
                Command::DrawLineRect { color, matrix } => handler.draw_line_rect(color, matrix),
                Command::PushMask => handler.push_mask(),
                Command::ActivateMask => handler.activate_mask(),
                Command::DeactivateMask => handler.deactivate_mask(),
                Command::PopMask => handler.pop_mask(),
                Command::Blend(commands, blend_mode) => handler.blend(commands, blend_mode),
            }
        }
    }

    pub fn drawing_mask(&self) -> bool {
        self.maskers_in_progress > 0
    }
}

impl CommandHandler for CommandList {
    #[inline]
    fn render_bitmap(
        &mut self,
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
    ) {
        if self.maskers_in_progress <= 1 {
            self.commands.push(Command::RenderBitmap {
                bitmap,
                transform,
                smoothing,
                pixel_snapping,
            });
        }
    }

    #[inline]
    fn render_stage3d(&mut self, bitmap: BitmapHandle, transform: Transform) {
        if self.maskers_in_progress <= 1 {
            self.commands
                .push(Command::RenderStage3D { bitmap, transform });
        }
    }

    #[inline]
    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform) {
        if self.maskers_in_progress <= 1 {
            self.commands
                .push(Command::RenderShape { shape, transform });
        }
    }

    #[inline]
    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        if self.maskers_in_progress <= 1 {
            self.commands.push(Command::DrawRect { color, matrix });
        }
    }

    #[inline]
    fn draw_line(&mut self, color: Color, matrix: Matrix) {
        if self.maskers_in_progress <= 1 {
            self.commands.push(Command::DrawLine { color, matrix });
        }
    }

    #[inline]
    fn draw_line_rect(&mut self, color: Color, matrix: Matrix) {
        if self.maskers_in_progress <= 1 {
            self.commands.push(Command::DrawLineRect { color, matrix });
        }
    }

    #[inline]
    fn push_mask(&mut self) {
        if self.maskers_in_progress == 0 {
            self.commands.push(Command::PushMask);
        }
        self.maskers_in_progress += 1;
    }

    #[inline]
    fn activate_mask(&mut self) {
        self.maskers_in_progress -= 1;
        if self.maskers_in_progress == 0 {
            self.commands.push(Command::ActivateMask);
        }
    }

    #[inline]
    fn deactivate_mask(&mut self) {
        if self.maskers_in_progress == 0 {
            self.commands.push(Command::DeactivateMask);
        }
        self.maskers_in_progress += 1;
    }

    #[inline]
    fn pop_mask(&mut self) {
        self.maskers_in_progress -= 1;
        if self.maskers_in_progress == 0 {
            self.commands.push(Command::PopMask);
        }
    }

    #[inline]
    fn blend(&mut self, commands: CommandList, blend_mode: RenderBlendMode) {
        if self.maskers_in_progress <= 1 {
            self.commands.push(Command::Blend(commands, blend_mode));
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    RenderBitmap {
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
        pixel_snapping: PixelSnapping,
    },
    RenderStage3D {
        bitmap: BitmapHandle,
        transform: Transform,
    },
    RenderShape {
        shape: ShapeHandle,
        transform: Transform,
    },
    DrawRect {
        color: Color,
        matrix: Matrix,
    },
    DrawLine {
        color: Color,
        matrix: Matrix,
    },
    DrawLineRect {
        color: Color,
        matrix: Matrix,
    },
    PushMask,
    ActivateMask,
    DeactivateMask,
    PopMask,
    Blend(CommandList, RenderBlendMode),
}
