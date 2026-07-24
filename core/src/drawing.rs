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
                DrawPath::Triangles { .. } => {
                    unreachable!("Static SWF shapes do not contain triangle paths")
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
        self.flush_current_path();
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

    fn flush_current_path(&mut self) {
        if let Some(existing) = self.current_fill.take().filter(|fill| {
            fill.commands
                .iter()
                .any(|command| !matches!(command, DrawCommand::MoveTo(_)))
        }) {
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

    pub fn draw_triangles(
        &mut self,
        vertices: Vec<Point<Twips>>,
        mut indices: Vec<u32>,
        texture_coords: Option<Vec<[f32; 3]>>,
    ) {
        let Some(fill) = &self.current_fill else {
            return;
        };
        let style = fill.style.clone();

        if matches!(style, FillStyle::Bitmap { .. })
            && let Some(texture_coords) = &texture_coords
        {
            assert_eq!(vertices.len(), texture_coords.len());
            let valid = |index: u32| {
                let coords = texture_coords[index as usize];
                coords.into_iter().all(f32::is_finite) && coords[2] != 0.0
            };
            if indices
                .as_chunks::<3>()
                .0
                .iter()
                .any(|triangle| !triangle.iter().copied().all(valid))
            {
                indices = indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .filter(|triangle| triangle.iter().copied().all(valid))
                    .flatten()
                    .copied()
                    .collect();
            }
        }
        if indices.is_empty() {
            return;
        }

        for &index in &indices {
            let point = vertices[index as usize];
            self.shape_bounds = self.shape_bounds.encompass(point);
            self.edge_bounds = self.edge_bounds.encompass(point);
        }

        self.paths.push(DrawingPath::Triangles {
            style,
            vertices,
            indices,
            texture_coords,
        });

        self.mark_dirty();
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
                    DrawingPath::Triangles {
                        style,
                        vertices,
                        indices,
                        texture_coords,
                    } => {
                        paths.push(DrawPath::Triangles {
                            style,
                            vertices: vertices.clone(),
                            indices: indices.clone(),
                            texture_coords: texture_coords.clone(),
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
            context
                .commands
                .render_shape(handle, context.transform_stack.transform());
        }
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
                DrawingPath::Triangles {
                    vertices, indices, ..
                } => {
                    for [i0, i1, i2] in indices.as_chunks::<3>().0 {
                        if point_in_triangle(
                            point,
                            vertices[*i0 as usize],
                            vertices[*i1 as usize],
                            vertices[*i2 as usize],
                        ) {
                            return true;
                        }
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
    Triangles {
        style: FillStyle,
        vertices: Vec<Point<Twips>>,
        indices: Vec<u32>,
        texture_coords: Option<Vec<[f32; 3]>>,
    },
}

fn point_in_triangle(
    point: Point<Twips>,
    a: Point<Twips>,
    b: Point<Twips>,
    c: Point<Twips>,
) -> bool {
    fn cross(a: Point<Twips>, b: Point<Twips>, point: Point<Twips>) -> i128 {
        let ab_x = i128::from(b.x.get()) - i128::from(a.x.get());
        let ab_y = i128::from(b.y.get()) - i128::from(a.y.get());
        let ap_x = i128::from(point.x.get()) - i128::from(a.x.get());
        let ap_y = i128::from(point.y.get()) - i128::from(a.y.get());
        ab_x * ap_y - ab_y * ap_x
    }

    let ab = cross(a, b, point);
    let bc = cross(b, c, point);
    let ca = cross(c, a, point);
    let has_negative = ab < 0 || bc < 0 || ca < 0;
    let has_positive = ab > 0 || bc > 0 || ca > 0;
    !has_negative || !has_positive
}

#[cfg(test)]
#[expect(clippy::items_after_test_module)]
mod tests {
    use super::{DrawCommand, Drawing, DrawingPath, point_in_triangle};
    use swf::{Color, FillStyle, LineStyle, Matrix, Point, Twips};

    #[test]
    fn triangle_hit_test_accepts_both_windings_and_edges() {
        let a = Point::from_pixels(0.0, 0.0);
        let b = Point::from_pixels(10.0, 0.0);
        let c = Point::from_pixels(0.0, 10.0);

        assert!(point_in_triangle(Point::from_pixels(2.0, 2.0), a, b, c));
        assert!(point_in_triangle(Point::from_pixels(2.0, 2.0), c, b, a));
        assert!(point_in_triangle(Point::from_pixels(5.0, 0.0), a, b, c));
        assert!(!point_in_triangle(Point::from_pixels(8.0, 8.0), a, b, c));
    }

    #[test]
    fn triangles_preserve_in_progress_drawing_state() {
        let mut drawing = Drawing::new();
        drawing.set_fill_style(Some(FillStyle::Color(Color::RED)));
        drawing.set_line_style(Some(LineStyle::new().with_width(Twips::ONE_PX)));
        drawing.draw_command(DrawCommand::MoveTo(Point::from_pixels(2.0, 3.0)));
        drawing.draw_command(DrawCommand::LineTo(Point::from_pixels(4.0, 5.0)));

        let cursor = drawing.cursor;
        let fill_start = drawing.fill_start;
        let fill_command_count = drawing.current_fill.as_ref().unwrap().commands.len();
        let line_command_count = drawing.current_line.as_ref().unwrap().commands.len();

        drawing.draw_triangles(
            vec![
                Point::from_pixels(0.0, 0.0),
                Point::from_pixels(10.0, 0.0),
                Point::from_pixels(0.0, 10.0),
                Point::from_pixels(1_000.0, 1_000.0),
            ],
            vec![0, 1, 2],
            None,
        );

        assert_eq!(drawing.cursor, cursor);
        assert_eq!(drawing.fill_start, fill_start);
        assert_eq!(
            drawing.current_fill.as_ref().unwrap().commands.len(),
            fill_command_count
        );
        assert_eq!(
            drawing.current_line.as_ref().unwrap().commands.len(),
            line_command_count
        );
        assert!(matches!(
            drawing.paths.last(),
            Some(DrawingPath::Triangles { indices, .. }) if indices == &[0, 1, 2]
        ));
        assert_eq!(drawing.self_bounds().x_max, Twips::from_pixels(10.0));
        assert_eq!(drawing.self_bounds().y_max, Twips::from_pixels(10.0));
        assert!(drawing.hit_test(
            Point::from_pixels(2.0, 2.0),
            &ruffle_render::matrix::Matrix::default()
        ));
        assert!(!drawing.hit_test(
            Point::from_pixels(8.0, 8.0),
            &ruffle_render::matrix::Matrix::default()
        ));
    }

    #[test]
    fn triangles_without_a_fill_do_not_draw() {
        let mut drawing = Drawing::new();
        drawing.draw_triangles(
            vec![
                Point::from_pixels(0.0, 0.0),
                Point::from_pixels(10.0, 0.0),
                Point::from_pixels(0.0, 10.0),
            ],
            vec![0, 1, 2],
            None,
        );

        assert!(drawing.is_empty);
        assert!(drawing.paths.is_empty());
    }

    #[test]
    fn bitmap_triangles_skip_invalid_texture_coordinates() {
        let mut drawing = Drawing::new();
        drawing.set_fill_style(Some(FillStyle::Bitmap {
            id: 0,
            matrix: Matrix::IDENTITY,
            is_smoothed: false,
            is_repeating: false,
        }));
        drawing.draw_triangles(
            vec![
                Point::from_pixels(0.0, 0.0),
                Point::from_pixels(10.0, 0.0),
                Point::from_pixels(0.0, 10.0),
            ],
            vec![0, 1, 2],
            Some(vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 1.0]]),
        );

        assert!(drawing.paths.is_empty());
    }
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
