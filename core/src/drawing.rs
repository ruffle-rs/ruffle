use crate::context::RenderContext;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, BitmapSize, BitmapSource};
use ruffle_render::commands::CommandHandler;
use ruffle_render::shape_utils::{
    DistilledShape, DrawCommand, DrawPath, FillRule, cubic_curve_bounds, quadratic_curve_bounds,
};
use std::cell::OnceCell;
use swf::{FillStyle, LineStyle, Point, Rectangle, Twips};

#[derive(Clone, Debug)]
pub struct Drawing {
    render_handle: OnceCell<ShapeHandle>,
    shape_bounds: Rectangle<Twips>,
    edge_bounds: Rectangle<Twips>,
    paths: Vec<DrawingPath>,
    bitmaps: Vec<BitmapInfo>,
    current_fill: Option<DrawingFill>,
    current_line: Option<DrawingLine>,
    pending_lines: Vec<DrawingLine>,
    cursor: Point<Twips>,
    fill_start: Point<Twips>,
    default_winding_rule: FillRule,
    /// Cached for fast emptiness check.
    is_empty: bool,
}

impl Default for Drawing {
    fn default() -> Self {
        Self::new()
    }
}

impl Drawing {
    pub fn new() -> Self {
        Self {
            render_handle: OnceCell::new(),
            shape_bounds: Default::default(),
            edge_bounds: Default::default(),
            paths: Vec::new(),
            bitmaps: Vec::new(),
            current_fill: None,
            current_line: None,
            pending_lines: Vec::new(),
            cursor: Point::ZERO,
            fill_start: Point::ZERO,
            default_winding_rule: FillRule::EvenOdd,
            is_empty: true,
        }
    }

    pub fn from_swf_shape(shape: &swf::Shape) -> Self {
        let mut this = Self {
            render_handle: OnceCell::new(),
            shape_bounds: shape.shape_bounds,
            edge_bounds: shape.edge_bounds,
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
            is_empty: true,
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

    fn mark_dirty(&mut self) {
        self.is_empty = false;
        self.render_handle.take();
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
        self.mark_dirty();
    }

    pub fn clear(&mut self) {
        self.current_fill = None;
        self.current_line = None;
        self.pending_lines.clear();
        self.paths.clear();
        self.bitmaps.clear();
        self.edge_bounds = Default::default();
        self.shape_bounds = Default::default();
        self.cursor = Point::ZERO;
        self.fill_start = Point::ZERO;
        self.is_empty = true;

        // An empty drawing doesn't need to hold onto a `ShapeHandle`.
        self.render_handle.take();
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

        self.mark_dirty();
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
        self.mark_dirty()
    }

    pub fn add_bitmap(&mut self, bitmap: BitmapInfo) -> u16 {
        let id = self.bitmaps.len() as u16;
        self.bitmaps.push(bitmap);
        id
    }

    /// Obtain a `ShapeHandle` that represents this `Drawing`, or `None` if it is empty.
    pub fn register_or_replace(&self, renderer: &mut dyn RenderBackend) -> Option<ShapeHandle> {
        if self.is_empty {
            return None;
        }

        let handle = self.render_handle.get_or_init(|| {
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
                shape_bounds: self.shape_bounds,
                edge_bounds: self.edge_bounds,
                id: 0,
            };
            renderer.register_shape(shape, self)
        });

        Some(handle.clone())
    }

    pub fn render(&self, context: &mut RenderContext) {
        if let Some(handle) = self.register_or_replace(context.renderer) {
            let mut transform = context.transform_stack.transform();
            // For drawings made entirely of 1-pixel orthogonal strokes (e.g. a
            // Spark `<s:Line>` separator, or any custom hairline divider),
            // snap the world translation down to the pixel grid with `floor`.
            // The tessellator pre-shifts these strokes by +0.5 px in the
            // perpendicular direction so the lyon quad falls on integer pixel
            // boundaries; if the parent layout places the host Sprite at a
            // fractional X (Spark groups frequently produce `X.5`), the
            // pre-shift combined with the fractional translation would slip
            // the quad back off the grid. Using `floor` (rather than `round`)
            // keeps the pixel position visually stable across fractions of
            // the same integer — a Sprite at `X.5` and one at `X.0` end up
            // activating the same pixel column.
            //
            // Only `tx`/`ty` are touched, and only when the matrix is a pure
            // translation (no scale, no rotation/skew): for transformed
            // shapes this snap wouldn't be visually meaningful.
            if self.is_orthogonal_hairline_only()
                && transform.matrix.a == 1.0
                && transform.matrix.d == 1.0
                && transform.matrix.b == 0.0
                && transform.matrix.c == 0.0
            {
                transform.matrix.tx = Twips::from_pixels(transform.matrix.tx.to_pixels().floor());
                transform.matrix.ty = Twips::from_pixels(transform.matrix.ty.to_pixels().floor());
            }
            context.commands.render_shape(handle, transform);
        }
    }

    /// Returns `true` if **any** path in the drawing is a 1-pixel-wide
    /// orthogonal (horizontal or vertical) stroke. Unlike
    /// [`Self::is_orthogonal_hairline_only`], this method tolerates fills or
    /// other stroke kinds coexisting with the hairline stroke — it only asks
    /// whether at least one such stroke is present. That is the precise
    /// condition under which `stretch_bounds` expands `shape_bounds` by
    /// ±0.5 px (the stroke radius for a 1 px stroke), producing a fractional
    /// `bounds.x_min`/`y_min` and, in the cacheAsBitmap path, a half-pixel
    /// shift of the whole subtree on the offscreen bitmap.
    pub fn contains_orthogonal_hairline_stroke(&self) -> bool {
        if self.is_empty {
            return false;
        }
        for path in &self.paths {
            if let DrawingPath::Line(line) = path
                && is_hairline_orthogonal_line(line)
            {
                return true;
            }
        }
        for line in &self.pending_lines {
            if is_hairline_orthogonal_line(line) {
                return true;
            }
        }
        if let Some(line) = &self.current_line
            && is_hairline_orthogonal_line(line)
        {
            return true;
        }
        false
    }

    /// Returns `true` if every path in the drawing is a 1-pixel-wide
    /// solid-color stroke whose segments are all horizontal or all vertical,
    /// with no fills, curves, or diagonals. These drawings benefit from
    /// snapping the world translation so the tessellator's pre-shifted quad
    /// lands on the pixel grid.
    fn is_orthogonal_hairline_only(&self) -> bool {
        if self.is_empty {
            return false;
        }
        if self.current_fill.is_some() {
            return false;
        }
        for path in &self.paths {
            match path {
                DrawingPath::Fill(_) => return false,
                DrawingPath::Line(line) => {
                    if !is_hairline_orthogonal_line(line) {
                        return false;
                    }
                }
            }
        }
        for line in &self.pending_lines {
            if !is_hairline_orthogonal_line(line) {
                return false;
            }
        }
        if let Some(line) = &self.current_line
            && !is_hairline_orthogonal_line(line)
        {
            return false;
        }
        true
    }

    pub fn self_bounds(&self) -> Rectangle<Twips> {
        self.shape_bounds
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
                    if shape_utils::draw_command_fill_hit_test(&fill.commands, fill.rule, point) {
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
        if let Some(fill) = &self.current_fill
            && shape_utils::draw_command_fill_hit_test(&fill.commands, fill.rule, point)
        {
            return true;
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
        if let Some(fill) = &mut self.current_fill
            && self.cursor != self.fill_start
        {
            fill.commands.push(DrawCommand::LineTo(self.fill_start));
            if let Some(line) = &mut self.current_line {
                line.commands.push(DrawCommand::LineTo(self.fill_start));
            }
            self.mark_dirty();
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

/// Returns `true` if the line is a 1-pixel-wide solid-color stroke whose
/// segments are all horizontal or all vertical. The SWF hairline default
/// (`width = 0` twips) rasterizes as 1-pixel-wide, so it qualifies.
fn is_hairline_orthogonal_line(line: &DrawingLine) -> bool {
    // 1 pixel = 20 twips. Include hairline strokes (width = 0 in SWF) up to and
    // including exactly 1 pixel.
    if line.style.width().get() > 20 {
        return false;
    }
    if !matches!(line.style.fill_style(), FillStyle::Color(_)) {
        return false;
    }
    let mut cursor: Option<Point<Twips>> = None;
    for cmd in &line.commands {
        match cmd {
            DrawCommand::MoveTo(p) => cursor = Some(*p),
            DrawCommand::LineTo(p) => {
                let from = match cursor {
                    Some(c) => c,
                    None => return false,
                };
                let dx = p.x.get() - from.x.get();
                let dy = p.y.get() - from.y.get();
                // Pure horizontal or pure vertical only — no diagonals.
                if dx != 0 && dy != 0 {
                    return false;
                }
                cursor = Some(*p);
            }
            DrawCommand::QuadraticCurveTo { .. } | DrawCommand::CubicCurveTo { .. } => {
                return false;
            }
        }
    }
    true
}

fn stretch_bounds(
    bounds: &Rectangle<Twips>,
    command: &DrawCommand,
    stroke_width: Twips,
    from: Point<Twips>,
) -> Rectangle<Twips> {
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
