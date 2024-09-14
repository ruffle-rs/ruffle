use crate::context::RenderContext;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, BitmapSize, BitmapSource};
use ruffle_render::commands::CommandHandler;
use ruffle_render::shape_utils::{
    cubic_curve_bounds, quadratic_curve_bounds, DistilledShape, DrawCommand, DrawPath, FillRule,
};
use std::cell::{Cell, RefCell};
use swf::{FillStyle, LineStyle, Point, Rectangle, Twips};

#[derive(Clone, Debug)]
pub struct Drawing {
    render_handle: RefCell<Option<ShapeHandle>>,
    shape_bounds: Rectangle<Twips>,
    edge_bounds: Rectangle<Twips>,
    dirty: Cell<bool>,
    paths: Vec<DrawingPath>,
    bitmaps: Vec<BitmapInfo>,
    current_fill: Option<DrawingFill>,
    current_line: Option<DrawingLine>,
    pending_lines: Vec<DrawingLine>,
    cursor: Point<Twips>,
    fill_start: Point<Twips>,
    default_winding_rule: FillRule,
}

impl Default for Drawing {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawing {
    pub fn new() -> Self {
        Self {
            render_handle: RefCell::new(None),
            shape_bounds: Default::default(),
            edge_bounds: Default::default(),
            dirty: Cell::new(false),
            paths: Vec::new(),
            bitmaps: Vec::new(),
            current_fill: None,
            current_line: None,
            pending_lines: Vec::new(),
            cursor: Point::ZERO,
            fill_start: Point::ZERO,
            default_winding_rule: FillRule::EvenOdd,
        }
    }

    pub fn from_swf_shape(shape: &swf::Shape) -> Self {
        let mut this = Self {
            render_handle: RefCell::new(None),
            shape_bounds: shape.shape_bounds.clone(),
            edge_bounds: shape.edge_bounds.clone(),
            dirty: Cell::new(true),
            paths: Vec::new(),
            bitmaps: Vec::new(),
            current_fill: None,
            current_line: None,
            pending_lines: Vec::new(),
            cursor: Point::ZERO,
            fill_start: Point::ZERO,
            default_winding_rule: if shape.flags.contains(swf::ShapeFlag::NON_ZERO_WINDING_RULE) {
                FillRule::NonZero
            } else {
                FillRule::EvenOdd
            },
        };

        let shape: DistilledShape = shape.into();
        for path in shape.paths {
            match path {
                DrawPath::Stroke {
                    style,
                    is_closed: _,
                    commands,
                } => {
                    this.set_line_style(Some(style.clone()));

                    for command in commands {
                        this.draw_command(command);
                    }

                    this.set_line_style(None);
                }
                DrawPath::Fill {
                    style,
                    commands,
                    winding_rule,
                } => {
                    this.new_fill(Some(style.clone()), Some(winding_rule));

                    for command in commands {
                        this.draw_command(command);
                    }

                    this.set_fill_style(None);
                }
            }
        }

        this
    }

    pub fn copy_from(&mut self, other: &Drawing) {
        *self = Drawing {
            render_handle: RefCell::new(None),
            dirty: Cell::new(true),
            shape_bounds: other.shape_bounds.clone(),
            edge_bounds: other.edge_bounds.clone(),
            paths: other.paths.clone(),
            bitmaps: other.bitmaps.clone(),
            current_fill: other.current_fill.clone(),
            current_line: other.current_line.clone(),
            pending_lines: other.pending_lines.clone(),
            cursor: other.cursor,
            fill_start: other.fill_start,
            default_winding_rule: other.default_winding_rule,
        }
    }

    /// Set fill style and reset fill rule to default.
    pub fn set_fill_style(&mut self, style: Option<FillStyle>) {
        self.new_fill(style, Some(self.default_winding_rule));
    }

    /// Set fill rule and keep the same fill style.
    pub fn set_fill_rule(&mut self, rule: Option<FillRule>) {
        let style = self.current_fill.as_ref().map(|fill| fill.style.clone());
        self.new_fill(style, rule);
    }

    /// Set fill style and rule.
    pub fn new_fill(&mut self, style: Option<FillStyle>, rule: Option<FillRule>) {
        self.close_path();
        if let Some(existing) = self.current_fill.take() {
            self.paths.push(DrawingPath::Fill(existing));
        }
        self.paths
            .extend(self.pending_lines.drain(..).map(DrawingPath::Line));
        if let Some(mut existing) = self.current_line.take() {
            existing.is_closed = self.cursor == self.fill_start;
            let style = existing.style.clone();
            self.paths.push(DrawingPath::Line(existing));
            self.current_line = Some(DrawingLine {
                style,
                commands: vec![DrawCommand::MoveTo(self.cursor)],
                is_closed: false,
            });
        }
        if let Some(style) = style {
            self.current_fill = Some(DrawingFill {
                style,
                rule: rule.unwrap_or(self.default_winding_rule),
                commands: vec![DrawCommand::MoveTo(self.cursor)],
            });
        }
        self.fill_start = self.cursor;
        self.dirty.set(true);
    }

    pub fn clear(&mut self) {
        self.current_fill = None;
        self.current_line = None;
        self.pending_lines.clear();
        self.paths.clear();
        self.bitmaps.clear();
        self.edge_bounds = Default::default();
        self.shape_bounds = Default::default();
        self.dirty.set(true);
        self.cursor = Point::ZERO;
        self.fill_start = Point::ZERO;
    }

    pub fn set_line_style(&mut self, style: Option<LineStyle>) {
        if let Some(mut existing) = self.current_line.take() {
            existing.is_closed = self.cursor == self.fill_start;
            if self.current_fill.is_some() {
                self.pending_lines.push(existing);
            } else {
                self.paths.push(DrawingPath::Line(existing));
            }
        }
        if let Some(style) = style {
            self.current_line = Some(DrawingLine {
                style,
                commands: vec![DrawCommand::MoveTo(self.cursor)],
                is_closed: false,
            });
        }

        self.dirty.set(true);
    }

    pub fn set_line_fill_style(&mut self, fill_style: FillStyle) {
        if let Some(style) = self.current_line.as_ref().map(|l| l.style.clone()) {
            self.set_line_style(Some(style.with_fill_style(fill_style)));
        }
    }

    pub fn draw_command(&mut self, command: DrawCommand) {
        let add_to_bounds = if let DrawCommand::MoveTo(move_to) = &command {
            // Close any pending fills before moving.
            self.close_path();
            self.fill_start = *move_to;
            false
        } else {
            true
        };

        // Add command to current fill.
        if let Some(fill) = &mut self.current_fill {
            fill.commands.push(command.clone());
        }
        // Add command to current line.
        let stroke_width = if let Some(line) = &mut self.current_line {
            line.commands.push(command.clone());
            line.style.width()
        } else {
            Twips::ZERO
        };

        // Expand bounds.
        if add_to_bounds {
            if self.fill_start == self.cursor {
                // If this is the initial command after a move, include the starting point.
                let command = DrawCommand::MoveTo(self.cursor);
                self.shape_bounds =
                    stretch_bounds(&self.shape_bounds, &command, stroke_width, self.cursor);
                self.edge_bounds =
                    stretch_bounds(&self.edge_bounds, &command, Twips::ZERO, self.cursor);
            }
            self.shape_bounds =
                stretch_bounds(&self.shape_bounds, &command, stroke_width, self.cursor);
            self.edge_bounds =
                stretch_bounds(&self.edge_bounds, &command, Twips::ZERO, self.cursor);
        }

        self.cursor = command.end_point();
        self.dirty.set(true);
    }

    pub fn add_bitmap(&mut self, bitmap: BitmapInfo) -> u16 {
        let id = self.bitmaps.len() as u16;
        self.bitmaps.push(bitmap);
        id
    }

    pub fn register_or_replace(&self, renderer: &mut dyn RenderBackend) -> ShapeHandle {
        if self.dirty.get() || self.render_handle.borrow().is_none() {
            self.dirty.set(false);
            let mut paths = Vec::with_capacity(self.paths.len());

            for path in &self.paths {
                match path {
                    DrawingPath::Fill(fill) => {
                        paths.push(DrawPath::Fill {
                            style: &fill.style,
                            commands: fill.commands.to_owned(),
                            winding_rule: fill.rule,
                        });
                    }
                    DrawingPath::Line(line) => {
                        paths.push(DrawPath::Stroke {
                            style: &line.style,
                            commands: line.commands.to_owned(),
                            is_closed: line.is_closed,
                        });
                    }
                }
            }

            if let Some(fill) = &self.current_fill {
                paths.push(DrawPath::Fill {
                    style: &fill.style,
                    commands: fill.commands.to_owned(),
                    winding_rule: fill.rule,
                })
            }

            for line in &self.pending_lines {
                let mut commands = line.commands.to_owned();
                let is_closed = if self.current_fill.is_some() {
                    commands.push(DrawCommand::LineTo(self.fill_start));
                    true
                } else {
                    self.cursor == self.fill_start
                };
                paths.push(DrawPath::Stroke {
                    style: &line.style,
                    commands,
                    is_closed,
                })
            }

            if let Some(line) = &self.current_line {
                let mut commands = line.commands.to_owned();
                let is_closed = if self.current_fill.is_some() {
                    commands.push(DrawCommand::LineTo(self.fill_start));
                    true
                } else {
                    self.cursor == self.fill_start
                };
                paths.push(DrawPath::Stroke {
                    style: &line.style,
                    commands,
                    is_closed,
                })
            }

            let shape = DistilledShape {
                paths,
                shape_bounds: self.shape_bounds.clone(),
                edge_bounds: self.edge_bounds.clone(),
                id: 0,
            };
            let handle = renderer.register_shape(shape, self);
            self.render_handle.replace(Some(handle.clone()));
            handle
        } else {
            self.render_handle
                .borrow()
                .to_owned()
                .expect("Render handle cannot be empty here, we guarded on is_none")
        }
    }

    pub fn render(&self, context: &mut RenderContext) {
        let handle = self.register_or_replace(context.renderer);
        context
            .commands
            .render_shape(handle, context.transform_stack.transform());
    }

    pub fn self_bounds(&self) -> &Rectangle<Twips> {
        &self.shape_bounds
    }

    pub fn hit_test(
        &self,
        point: Point<Twips>,
        local_matrix: &ruffle_render::matrix::Matrix,
    ) -> bool {
        use ruffle_render::shape_utils;
        for path in &self.paths {
            match path {
                DrawingPath::Fill(fill) => {
                    if shape_utils::draw_command_fill_hit_test(&fill.commands, point) {
                        return true;
                    }
                }
                DrawingPath::Line(line) => {
                    if shape_utils::draw_command_stroke_hit_test(
                        &line.commands,
                        line.style.width(),
                        point,
                        local_matrix,
                    ) {
                        return true;
                    }
                }
            }
        }

        // The pending fill will auto-close.
        if let Some(fill) = &self.current_fill {
            if shape_utils::draw_command_fill_hit_test(&fill.commands, point) {
                return true;
            }
        }

        for line in &self.pending_lines {
            if shape_utils::draw_command_stroke_hit_test(
                &line.commands,
                line.style.width(),
                point,
                local_matrix,
            ) {
                return true;
            }
        }

        if let Some(line) = &self.current_line {
            if shape_utils::draw_command_stroke_hit_test(
                &line.commands,
                line.style.width(),
                point,
                local_matrix,
            ) {
                return true;
            }

            // Stroke auto-closes if part of a fill; also check the closing line segment.
            if self.current_fill.is_some()
                && self.cursor != self.fill_start
                && shape_utils::draw_command_stroke_hit_test(
                    &[
                        DrawCommand::MoveTo(self.cursor),
                        DrawCommand::LineTo(self.fill_start),
                    ],
                    line.style.width(),
                    point,
                    local_matrix,
                )
            {
                return true;
            }
        }

        false
    }

    // Ensures that the path is closed for a pending fill.
    pub fn close_path(&mut self) {
        if let Some(fill) = &mut self.current_fill {
            if self.cursor != self.fill_start {
                fill.commands.push(DrawCommand::LineTo(self.fill_start));
                if let Some(line) = &mut self.current_line {
                    line.commands.push(DrawCommand::LineTo(self.fill_start));
                }
                self.dirty.set(true);
            }
        }
    }
}

impl BitmapSource for Drawing {
    fn bitmap_size(&self, id: u16) -> Option<BitmapSize> {
        self.bitmaps.get(id as usize).map(|bm| BitmapSize {
            width: bm.width,
            height: bm.height,
        })
    }
    fn bitmap_handle(&self, id: u16, _backend: &mut dyn RenderBackend) -> Option<BitmapHandle> {
        self.bitmaps.get(id as usize).map(|bm| bm.handle.clone())
    }
}

#[derive(Debug, Clone)]
struct DrawingFill {
    style: FillStyle,
    rule: FillRule,
    commands: Vec<DrawCommand>,
}

#[derive(Debug, Clone)]
struct DrawingLine {
    style: LineStyle,
    commands: Vec<DrawCommand>,
    is_closed: bool,
}

#[derive(Debug, Clone)]
enum DrawingPath {
    Fill(DrawingFill),
    Line(DrawingLine),
}

fn stretch_bounds(
    bounds: &Rectangle<Twips>,
    command: &DrawCommand,
    stroke_width: Twips,
    from: Point<Twips>,
) -> Rectangle<Twips> {
    let bounds = bounds.clone();
    match *command {
        DrawCommand::MoveTo(point) | DrawCommand::LineTo(point) => {
            let radius = stroke_width / 2;
            bounds
                .encompass(Point::new(point.x - radius, point.y - radius))
                .encompass(Point::new(point.x + radius, point.y + radius))
        }
        DrawCommand::QuadraticCurveTo { control, anchor } => {
            bounds.union(&quadratic_curve_bounds(from, stroke_width, control, anchor))
        }
        DrawCommand::CubicCurveTo {
            control_a,
            control_b,
            anchor,
        } => bounds.union(&cubic_curve_bounds(
            from,
            stroke_width,
            control_a,
            control_b,
            anchor,
        )),
    }
}
