use crate::backend::render::ShapeHandle;
use crate::bounding_box::BoundingBox;
use crate::context::RenderContext;
use crate::shape_utils::{DistilledShape, DrawCommand, DrawPath};
use gc_arena::Collect;
use std::cell::Cell;
use swf::{FillStyle, LineStyle, Twips};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct Drawing {
    render_handle: Cell<Option<ShapeHandle>>,
    shape_bounds: BoundingBox,
    edge_bounds: BoundingBox,
    dirty: Cell<bool>,
    fills: Vec<(FillStyle, Vec<DrawCommand>)>,
    lines: Vec<(LineStyle, Vec<DrawCommand>)>,
    current_fill: Option<(FillStyle, Vec<DrawCommand>)>,
    current_line: Option<(LineStyle, Vec<DrawCommand>)>,
    cursor: (Twips, Twips),
}

impl Drawing {
    pub fn new() -> Self {
        Self {
            render_handle: Cell::new(None),
            shape_bounds: BoundingBox::default(),
            edge_bounds: BoundingBox::default(),
            dirty: Cell::new(false),
            fills: Vec::new(),
            lines: Vec::new(),
            current_fill: None,
            current_line: None,
            cursor: (Twips::zero(), Twips::zero()),
        }
    }

    pub fn set_fill_style(&mut self, style: Option<FillStyle>) {
        // TODO: If current_fill is not closed, we should close it and also close current_line

        if let Some(existing) = self.current_fill.take() {
            self.fills.push(existing);
        }
        if let Some(style) = style {
            self.current_fill = Some((
                style,
                vec![DrawCommand::MoveTo {
                    x: self.cursor.0,
                    y: self.cursor.1,
                }],
            ));
        }

        self.dirty.set(true);
    }

    pub fn clear(&mut self) {
        self.current_fill = None;
        self.current_line = None;
        self.fills.clear();
        self.lines.clear();
        self.edge_bounds = BoundingBox::default();
        self.shape_bounds = BoundingBox::default();
        self.dirty.set(true);
        self.cursor = (Twips::zero(), Twips::zero());
    }

    pub fn set_line_style(&mut self, style: Option<LineStyle>) {
        if let Some(existing) = self.current_line.take() {
            self.lines.push(existing);
        }
        if let Some(style) = style {
            self.current_line = Some((
                style,
                vec![DrawCommand::MoveTo {
                    x: self.cursor.0,
                    y: self.cursor.1,
                }],
            ));
        }

        self.dirty.set(true);
    }

    pub fn draw_command(&mut self, command: DrawCommand) {
        let mut include_last = false;
        let stroke_width = if let Some((style, _)) = &self.current_line {
            style.width
        } else {
            Twips::zero()
        };

        match command {
            DrawCommand::MoveTo { .. } => {}
            DrawCommand::LineTo { .. } => {
                stretch_bounding_box(&mut self.shape_bounds, &command, stroke_width);
                stretch_bounding_box(&mut self.edge_bounds, &command, Twips::zero());
                include_last = true;
            }
            DrawCommand::CurveTo { .. } => {
                stretch_bounding_box(&mut self.shape_bounds, &command, stroke_width);
                stretch_bounding_box(&mut self.edge_bounds, &command, Twips::zero());
                include_last = true;
            }
        }

        self.cursor = command.end_point();

        if let Some((_, commands)) = &mut self.current_line {
            commands.push(command.clone());
        }
        if let Some((_, commands)) = &mut self.current_fill {
            commands.push(command);
        }

        if include_last {
            if let Some(command) = self
                .current_fill
                .as_ref()
                .and_then(|(_, commands)| commands.last())
            {
                stretch_bounding_box(&mut self.shape_bounds, command, stroke_width);
                stretch_bounding_box(&mut self.edge_bounds, command, Twips::zero());
            }

            if let Some(command) = self
                .current_line
                .as_ref()
                .and_then(|(_, commands)| commands.last())
            {
                stretch_bounding_box(&mut self.shape_bounds, command, stroke_width);
                stretch_bounding_box(&mut self.edge_bounds, command, Twips::zero());
            }
        }

        self.dirty.set(true);
    }

    pub fn render(&self, context: &mut RenderContext) {
        if self.dirty.get() {
            self.dirty.set(false);
            let mut paths = Vec::new();

            for (style, commands) in &self.fills {
                paths.push(DrawPath::Fill {
                    style,
                    commands: commands.to_owned(),
                })
            }

            // TODO: If the current_fill is not closed, we should automatically close current_line

            if let Some((style, commands)) = &self.current_fill {
                paths.push(DrawPath::Fill {
                    style,
                    commands: commands.to_owned(),
                })
            }

            for (style, commands) in &self.lines {
                paths.push(DrawPath::Stroke {
                    style,
                    commands: commands.to_owned(),
                    is_closed: false, // TODO: Determine this
                })
            }

            if let Some((style, commands)) = &self.current_line {
                paths.push(DrawPath::Stroke {
                    style,
                    commands: commands.to_owned(),
                    is_closed: false, // TODO: Determine this
                })
            }

            let shape = DistilledShape {
                paths,
                shape_bounds: self.shape_bounds.clone(),
                edge_bounds: self.edge_bounds.clone(),
                id: 0,
            };

            if let Some(handle) = self.render_handle.get() {
                context.renderer.replace_shape(shape, handle);
            } else {
                self.render_handle
                    .set(Some(context.renderer.register_shape(shape)));
            }
        }

        if let Some(handle) = self.render_handle.get() {
            context
                .renderer
                .render_shape(handle, context.transform_stack.transform());
        }
    }

    pub fn self_bounds(&self) -> BoundingBox {
        self.shape_bounds.clone()
    }
}

fn stretch_bounding_box(
    bounding_box: &mut BoundingBox,
    command: &DrawCommand,
    stroke_width: Twips,
) {
    let radius = stroke_width / 2;
    match *command {
        DrawCommand::MoveTo { x, y } => {
            bounding_box.encompass(x - radius, y - radius);
            bounding_box.encompass(x + radius, y + radius);
        }
        DrawCommand::LineTo { x, y } => {
            bounding_box.encompass(x - radius, y - radius);
            bounding_box.encompass(x + radius, y + radius);
        }
        DrawCommand::CurveTo { x1, y1, x2, y2 } => {
            bounding_box.encompass(x1 - radius, y1 - radius);
            bounding_box.encompass(x1 + radius, y1 + radius);
            bounding_box.encompass(x2 - radius, y2 - radius);
            bounding_box.encompass(x2 + radius, y2 + radius);
        }
    }
}
