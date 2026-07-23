use crate::context::RenderContext;
use ruffle_render::backend::{RenderBackend, ShapeHandle};
use ruffle_render::bitmap::{BitmapHandle, BitmapInfo, BitmapSize, BitmapSource};
use ruffle_render::commands::CommandHandler;
use ruffle_render::matrix::Matrix;
use ruffle_render::shape_utils::{
    DistilledShape, DrawCommand, DrawPath, FillRule, cubic_curve_bounds, quadratic_curve_bounds,
};
use std::cell::OnceCell;
use swf::{FillStyle, LineStyle, Point, Rectangle, Twips};

#[derive(Clone, Debug)]
pub struct Drawing {
    render_handle: OnceCell<ShapeHandle>,
    /// Cached partition of this drawing into runs of consecutive paths, used
    /// by [`Self::render`]. Runs made entirely of 1px orthogonal strokes are
    /// registered as separate shapes so they can be snapped to the pixel
    /// grid at render time (see [`Self::build_render_units`]); everything
    /// else renders with the unmodified world transform. Invalidated by
    /// `mark_dirty`, exactly like `render_handle`.
    render_units: OnceCell<Vec<RenderUnit>>,
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
            render_units: OnceCell::new(),
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
            render_units: OnceCell::new(),
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
        self.render_units.take();
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
        self.render_units.take();
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

    /// Builds the finalized `DrawPath` list for this drawing, in paint order:
    /// committed paths first, then the still-open fill and lines (with their
    /// auto-close segment). This is the single source of truth for what gets
    /// rendered, shared by [`Self::register_or_replace`] (one shape with
    /// everything) and [`Self::build_render_units`] (the same paths,
    /// partitioned into snappable runs).
    fn distilled_paths(&self) -> Vec<DrawPath<'_>> {
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

        paths
    }

    /// Obtain a `ShapeHandle` that represents this `Drawing`, or `None` if it is empty.
    pub fn register_or_replace(&self, renderer: &mut dyn RenderBackend) -> Option<ShapeHandle> {
        if self.is_empty {
            return None;
        }

        let handle = self.render_handle.get_or_init(|| {
            let shape = DistilledShape {
                paths: self.distilled_paths(),
                shape_bounds: self.shape_bounds,
                edge_bounds: self.edge_bounds,
                id: 0,
            };
            renderer.register_shape(shape, self)
        });

        Some(handle.clone())
    }

    /// Partitions the drawing into runs of consecutive paths and registers
    /// one shape per run.
    ///
    /// Paint order is preserved: runs are strictly consecutive slices of the
    /// path list, rendered in the original order, so overlapping geometry
    /// composites exactly as with a single shape. A new run starts whenever a
    /// path's snap profile is incompatible with the current run's: qualifying
    /// hairline strokes with the same sub-pixel phase accumulate into one
    /// snappable run, while everything else (fills, wide or diagonal strokes,
    /// gradient strokes) accumulates into plain runs that render with the
    /// untouched world transform.
    ///
    /// In the common cases this degenerates gracefully: a drawing with no
    /// qualifying stroke produces a single plain run (identical to the
    /// pre-existing single-shape path), and a Flex grid whose separators sit
    /// on integer coordinates produces at most a couple of runs.
    fn build_render_units(&self, renderer: &mut dyn RenderBackend) -> Vec<RenderUnit> {
        let mut runs: Vec<(Vec<DrawPath<'_>>, Option<SnapParams>)> = Vec::new();

        for path in self.distilled_paths() {
            let snap = match &path {
                DrawPath::Stroke {
                    style, commands, ..
                } => hairline_snap_profile(style, commands),
                DrawPath::Fill { .. } => None,
            };

            if let Some((run_paths, run_snap)) = runs.last_mut() {
                let merged = match (&run_snap, &snap) {
                    (None, None) => Some(None),
                    (Some(a), Some(b)) => a.try_merge(b).map(Some),
                    _ => None,
                };
                if let Some(merged) = merged {
                    *run_snap = merged;
                    run_paths.push(path);
                    continue;
                }
            }
            runs.push((vec![path], snap));
        }

        runs.into_iter()
            .map(|(paths, snap)| {
                let shape = DistilledShape {
                    paths,
                    // Every run reuses the whole drawing's bounds: bounds are
                    // not part of the tessellated geometry, and keeping them
                    // identical avoids re-deriving per-run boxes.
                    shape_bounds: self.shape_bounds,
                    edge_bounds: self.edge_bounds,
                    id: 0,
                };
                RenderUnit {
                    handle: renderer.register_shape(shape, self),
                    snap,
                }
            })
            .collect()
    }

    pub fn render(&self, context: &mut RenderContext) {
        if self.is_empty {
            return;
        }

        let units = self
            .render_units
            .get_or_init(|| self.build_render_units(context.renderer));

        let base_transform = context.transform_stack.transform();
        for unit in units {
            let mut transform = base_transform.clone();
            if let Some(snap) = unit.snap {
                snap_hairline_run(&mut transform.matrix, snap);
            }
            context
                .commands
                .render_shape(unit.handle.clone(), transform);
        }
    }

    /// Returns `true` if **any** stroke in the drawing is a 1-pixel-wide
    /// orthogonal (horizontal or vertical) solid-color stroke. Fills or other
    /// stroke kinds may coexist — this only asks whether at least one such
    /// stroke is present. That is the precise condition under which
    /// `stretch_bounds` expands `shape_bounds` by the half-pixel stroke
    /// radius, producing a fractional `bounds.x_min`/`y_min` and, in the
    /// cacheAsBitmap path, a half-pixel shift of the whole subtree on the
    /// offscreen bitmap (see `render_base`).
    pub fn contains_orthogonal_hairline_stroke(&self) -> bool {
        if self.is_empty {
            return false;
        }
        fn is_hairline(line: &DrawingLine) -> bool {
            is_orthogonal_hairline_stroke(&line.style, &line.commands)
        }
        self.paths
            .iter()
            .any(|path| matches!(path, DrawingPath::Line(line) if is_hairline(line)))
            || self.pending_lines.iter().any(is_hairline)
            || self.current_line.as_ref().is_some_and(is_hairline)
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

/// One registered slice of a [`Drawing`], produced by
/// [`Drawing::build_render_units`]: a run of consecutive paths sharing the
/// same snapping treatment.
#[derive(Debug, Clone)]
struct RenderUnit {
    handle: ShapeHandle,
    /// `Some` when this run is made entirely of 1px orthogonal strokes that
    /// can be snapped to the pixel grid with a single translation.
    snap: Option<SnapParams>,
}

/// Everything needed to compute, at render time, the screen-space translation
/// that puts a run of 1px orthogonal strokes exactly on the pixel grid.
///
/// A 1px stroke drawn along an axis covers `center ± 0.5px` in the
/// perpendicular direction. It rasterizes as one crisp pixel column/row only
/// when `center` lands on a half-pixel (`N + 0.5`); anywhere else the
/// anti-aliasing smears it across two pixels — the "blurry separator" bug.
/// Because every same-orientation segment in the run shares the same
/// sub-pixel phase (guaranteed by [`SnapParams::try_merge`] /
/// [`hairline_snap_profile`]), one sample coordinate per axis is enough to
/// derive a translation that aligns the whole run at once.
#[derive(Debug, Clone, Copy)]
struct SnapParams {
    /// Local x of one vertical segment, when the run contains vertical
    /// strokes.
    sample_x: Option<Twips>,
    /// Local y of one horizontal segment, when the run contains horizontal
    /// strokes.
    sample_y: Option<Twips>,
}

impl SnapParams {
    /// Merges the profiles of two paths destined for the same run, or `None`
    /// when a single translation cannot align both (their perpendicular
    /// coordinates sit at different sub-pixel phases).
    fn try_merge(&self, other: &SnapParams) -> Option<SnapParams> {
        fn axis(a: Option<Twips>, b: Option<Twips>) -> Result<Option<Twips>, ()> {
            match (a, b) {
                (Some(a), Some(b)) if pixel_phase(a) != pixel_phase(b) => Err(()),
                _ => Ok(a.or(b)),
            }
        }
        Some(SnapParams {
            sample_x: axis(self.sample_x, other.sample_x).ok()?,
            sample_y: axis(self.sample_y, other.sample_y).ok()?,
        })
    }
}

const TWIPS_PER_PIXEL: i32 = 20;

/// Sub-pixel phase of a coordinate: its offset within the pixel, in twips
/// (`0..20`). Two parallel strokes with equal phase are aligned by the same
/// translation.
fn pixel_phase(t: Twips) -> i32 {
    t.get().rem_euclid(TWIPS_PER_PIXEL)
}

/// Classifies a stroke path for pixel-grid snapping.
///
/// A path qualifies when it is a solid-color stroke at most 1px wide (the SWF
/// hairline, width 0, rasterizes 1px wide and qualifies too) whose segments
/// are all axis-aligned, and the perpendicular coordinates of
/// same-orientation segments all share the same sub-pixel phase — so a single
/// translation aligns every segment at once. Paths mixing both orientations
/// (e.g. a `drawRect` outline) qualify as long as each axis is internally
/// consistent; the returned profile then snaps both axes.
///
/// Returns `None` for anything else — curves, diagonals, wider strokes,
/// gradient/bitmap strokes, phase-mismatched segments — leaving those paths
/// to render exactly as before.
fn hairline_snap_profile(style: &LineStyle, commands: &[DrawCommand]) -> Option<SnapParams> {
    if style.width().get() > TWIPS_PER_PIXEL {
        return None;
    }
    if !matches!(style.fill_style(), FillStyle::Color(_)) {
        return None;
    }

    let mut sample_x: Option<Twips> = None;
    let mut sample_y: Option<Twips> = None;
    let mut cursor: Option<Point<Twips>> = None;
    for command in commands {
        match command {
            DrawCommand::MoveTo(p) => cursor = Some(*p),
            DrawCommand::LineTo(p) => {
                let from = cursor?;
                let dx = p.x - from.x;
                let dy = p.y - from.y;
                if dx != Twips::ZERO && dy != Twips::ZERO {
                    // Diagonal segment — there is no pixel-aligned target.
                    return None;
                }
                if dy != Twips::ZERO {
                    // Vertical segment at x = from.x.
                    match sample_x {
                        None => sample_x = Some(from.x),
                        Some(sx) if pixel_phase(sx) == pixel_phase(from.x) => {}
                        Some(_) => return None,
                    }
                }
                if dx != Twips::ZERO {
                    // Horizontal segment at y = from.y.
                    match sample_y {
                        None => sample_y = Some(from.y),
                        Some(sy) if pixel_phase(sy) == pixel_phase(from.y) => {}
                        Some(_) => return None,
                    }
                }
                cursor = Some(*p);
            }
            DrawCommand::QuadraticCurveTo { .. } | DrawCommand::CubicCurveTo { .. } => {
                return None;
            }
        }
    }

    if sample_x.is_none() && sample_y.is_none() {
        // No actual segments (only moves) — nothing to snap.
        return None;
    }
    Some(SnapParams { sample_x, sample_y })
}

/// Permissive variant of [`hairline_snap_profile`] used to gate the
/// cacheAsBitmap bounds snap in `render_base`: it only asks whether the path
/// is a 1px orthogonal solid-color stroke, with no requirement that segments
/// share a sub-pixel phase (any such stroke expands the drawing bounds by the
/// half-pixel radius, phase-consistent or not).
fn is_orthogonal_hairline_stroke(style: &LineStyle, commands: &[DrawCommand]) -> bool {
    if style.width().get() > TWIPS_PER_PIXEL {
        return false;
    }
    if !matches!(style.fill_style(), FillStyle::Color(_)) {
        return false;
    }
    let mut cursor: Option<Point<Twips>> = None;
    for command in commands {
        match command {
            DrawCommand::MoveTo(p) => cursor = Some(*p),
            DrawCommand::LineTo(p) => {
                let Some(from) = cursor else {
                    return false;
                };
                if p.x != from.x && p.y != from.y {
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

/// Adjusts the world translation so a snappable stroke run lands exactly on
/// the pixel grid.
///
/// The snap happens here — in screen space, with the final world matrix in
/// hand — and not at tessellation time: the tessellated geometry is in path
/// space, and any path-space adjustment is defeated by whatever fractional
/// translation the container hierarchy contributes afterwards. Per-axis, the
/// screen-space stroke center is snapped to the half-pixel of the pixel that
/// contains it (majority coverage): the stroke quad `center ± 0.5px` then
/// covers exactly one pixel column/row, which rasterizes crisp under plain
/// AA and MSAA alike, and never moves the stroke by more than half a pixel.
///
/// Snapping is only meaningful when the shape reaches the screen unscaled
/// and unrotated. The gate tolerates the tiny scale drift that Flex
/// percent-based layouts accumulate through matrix concatenation (observed
/// around 1e-7; anything below `SNAP_EPS` is visually irrelevant), and the
/// center is computed with the actual matrix coefficients so the tolerated
/// drift does not bias the result. Scaled or rotated shapes render exactly
/// as before.
fn snap_hairline_run(matrix: &mut Matrix, snap: SnapParams) {
    const SNAP_EPS: f32 = 1e-5;
    if (matrix.a - 1.0).abs() > SNAP_EPS
        || (matrix.d - 1.0).abs() > SNAP_EPS
        || matrix.b.abs() > SNAP_EPS
        || matrix.c.abs() > SNAP_EPS
    {
        return;
    }

    const PX: f64 = TWIPS_PER_PIXEL as f64;
    if let Some(sx) = snap.sample_x {
        let center = matrix.a as f64 * sx.get() as f64 + matrix.tx.get() as f64;
        let snapped = (center / PX).floor() * PX + PX / 2.0;
        matrix.tx += Twips::new((snapped - center).round() as i32);
    }
    if let Some(sy) = snap.sample_y {
        let center = matrix.d as f64 * sy.get() as f64 + matrix.ty.get() as f64;
        let snapped = (center / PX).floor() * PX + PX / 2.0;
        matrix.ty += Twips::new((snapped - center).round() as i32);
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
