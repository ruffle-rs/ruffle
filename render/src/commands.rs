use crate::backend::ShapeHandle;
use crate::bitmap::BitmapHandle;
use crate::matrix::Matrix;
use crate::transform::Transform;
use swf::{BlendMode, Color};

pub trait CommandHandler {
    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: Transform, smoothing: bool);
    fn render_stage3d(&mut self, bitmap: BitmapHandle, transform: Transform);
    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform, is_stroke: bool);
    fn draw_rect(&mut self, color: Color, matrix: Matrix);
    fn push_mask(&mut self);
    fn activate_mask(&mut self);
    fn deactivate_mask(&mut self);
    fn pop_mask(&mut self);

    fn blend(&mut self, commands: CommandList, blend_mode: BlendMode);
}

#[derive(Debug, Default, Clone)]
pub struct CommandList {
    pub commands: Vec<Command>,
}

impl CommandList {
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
                } => handler.render_bitmap(bitmap, transform, smoothing),
                Command::RenderStage3D { bitmap, transform } => {
                    handler.render_stage3d(bitmap, transform)
                }
                Command::RenderShape {
                    shape,
                    transform,
                    is_stroke,
                } => handler.render_shape(shape, transform, is_stroke),
                Command::DrawRect { color, matrix } => handler.draw_rect(color, matrix),
                Command::PushMask => handler.push_mask(),
                Command::ActivateMask => handler.activate_mask(),
                Command::DeactivateMask => handler.deactivate_mask(),
                Command::PopMask => handler.pop_mask(),
                Command::Blend(commands, blend_mode) => handler.blend(commands, blend_mode),
            }
        }
    }
}

impl CommandHandler for CommandList {
    fn render_bitmap(&mut self, bitmap: BitmapHandle, transform: Transform, smoothing: bool) {
        self.commands.push(Command::RenderBitmap {
            bitmap,
            transform,
            smoothing,
        });
    }

    fn render_stage3d(&mut self, bitmap: BitmapHandle, transform: Transform) {
        self.commands
            .push(Command::RenderStage3D { bitmap, transform });
    }

    fn render_shape(&mut self, shape: ShapeHandle, transform: Transform, is_stroke: bool) {
        self.commands.push(Command::RenderShape {
            shape,
            transform,
            is_stroke,
        });
    }

    fn draw_rect(&mut self, color: Color, matrix: Matrix) {
        self.commands.push(Command::DrawRect { color, matrix });
    }

    fn push_mask(&mut self) {
        self.commands.push(Command::PushMask);
    }

    fn activate_mask(&mut self) {
        self.commands.push(Command::ActivateMask);
    }

    fn deactivate_mask(&mut self) {
        self.commands.push(Command::DeactivateMask);
    }

    fn pop_mask(&mut self) {
        self.commands.push(Command::PopMask);
    }

    fn blend(&mut self, commands: CommandList, blend_mode: BlendMode) {
        self.commands.push(Command::Blend(commands, blend_mode));
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    RenderBitmap {
        bitmap: BitmapHandle,
        transform: Transform,
        smoothing: bool,
    },
    RenderStage3D {
        bitmap: BitmapHandle,
        transform: Transform,
    },
    RenderShape {
        shape: ShapeHandle,
        transform: Transform,
        is_stroke: bool,
    },
    DrawRect {
        color: Color,
        matrix: Matrix,
    },
    PushMask,
    ActivateMask,
    DeactivateMask,
    PopMask,
    Blend(CommandList, BlendMode),
}
