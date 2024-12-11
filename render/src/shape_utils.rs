use crate::matrix::Matrix;
use enum_map::Enum;
use smallvec::SmallVec;
use swf::{CharacterId, FillStyle, LineStyle, Rectangle, Shape, ShapeRecord, Twips};

/// Controls the accuracy of the approximated quadratic curve, when splitting up a cubic curve
const CUBIC_CURVE_TOLERANCE: f64 = 0.01;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FillRule {
    EvenOdd,
    NonZero,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Enum, Hash)]
pub enum GradientType {
    Linear,
    Radial,
    Focal,
}

#[cfg(feature = "tessellator")]
impl From<FillRule> for lyon::path::FillRule {
    fn from(rule: FillRule) -> lyon::path::FillRule {
        match rule {
            FillRule::EvenOdd => lyon::path::FillRule::EvenOdd,
            FillRule::NonZero => lyon::path::FillRule::NonZero,
        }
    }
}

pub fn calculate_shape_bounds(shape_records: &[swf::ShapeRecord]) -> swf::Rectangle<Twips> {
    let mut bounds = swf::Rectangle {
        x_min: Twips::new(i32::MAX),
        y_min: Twips::new(i32::MAX),
        x_max: Twips::new(i32::MIN),
        y_max: Twips::new(i32::MIN),
    };
    let mut cursor = swf::Point::ZERO;
    for record in shape_records {
        match record {
            swf::ShapeRecord::StyleChange(style_change) => {
                if let Some(move_to) = &style_change.move_to {
                    cursor = *move_to;
                    bounds.x_min = bounds.x_min.min(cursor.x);
                    bounds.x_max = bounds.x_max.max(cursor.x);
                    bounds.y_min = bounds.y_min.min(cursor.y);
                    bounds.y_max = bounds.y_max.max(cursor.y);
                }
            }
            swf::ShapeRecord::StraightEdge { delta } => {
                cursor += *delta;
                bounds.x_min = bounds.x_min.min(cursor.x);
                bounds.x_max = bounds.x_max.max(cursor.x);
                bounds.y_min = bounds.y_min.min(cursor.y);
                bounds.y_max = bounds.y_max.max(cursor.y);
            }
            swf::ShapeRecord::CurvedEdge {
                control_delta,
                anchor_delta,
            } => {
                cursor += *control_delta;
                let control = cursor;
                cursor += *anchor_delta;
                let anchor = cursor;
                bounds = bounds.union(&quadratic_curve_bounds(
                    cursor,
                    Twips::ZERO,
                    control,
                    anchor,
                ));
            }
        }
    }
    if bounds.x_max < bounds.x_min || bounds.y_max < bounds.y_min {
        bounds = Default::default();
    }
    bounds
}

/// `DrawPath` represents a solid fill or a stroke.
/// Fills are always closed paths, while strokes may be open or closed.
/// Closed paths will have the first point equal to the last point.
#[derive(Clone, Debug)]
pub enum DrawPath<'a> {
    Stroke {
        style: &'a LineStyle,
        is_closed: bool,
        commands: Vec<DrawCommand>,
    },
    Fill {
        style: &'a FillStyle,
        commands: Vec<DrawCommand>,
        winding_rule: FillRule,
    },
}

/// `DistilledShape` represents a ready-to-be-consumed collection of paths (both fills and strokes)
/// that has been converted down from another source (such as SWF's `swf::Shape` format).
#[derive(Clone, Debug)]
pub struct DistilledShape<'a> {
    pub paths: Vec<DrawPath<'a>>,
    pub shape_bounds: Rectangle<Twips>,
    pub edge_bounds: Rectangle<Twips>,
    pub id: CharacterId,
}

impl<'a> From<&'a swf::Shape> for DistilledShape<'a> {
    fn from(shape: &'a Shape) -> Self {
        Self {
            paths: ShapeConverter::from_shape(shape).into_commands(),
            shape_bounds: shape.shape_bounds.clone(),
            edge_bounds: shape.edge_bounds.clone(),
            id: shape.id,
        }
    }
}

/// `DrawCommands` trace the outline of a path.
/// Fills follow the even-odd fill rule, with opposite winding for holes.
#[derive(Clone, Debug)]
pub enum DrawCommand {
    MoveTo(swf::Point<Twips>),
    LineTo(swf::Point<Twips>),
    QuadraticCurveTo {
        control: swf::Point<Twips>,
        anchor: swf::Point<Twips>,
    },
    CubicCurveTo {
        control_a: swf::Point<Twips>,
        control_b: swf::Point<Twips>,
        anchor: swf::Point<Twips>,
    },
}

impl DrawCommand {
    pub fn end_point(&self) -> swf::Point<Twips> {
        match self {
            DrawCommand::MoveTo(point)
            | DrawCommand::LineTo(point)
            | DrawCommand::QuadraticCurveTo { anchor: point, .. }
            | DrawCommand::CubicCurveTo { anchor: point, .. } => *point,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: Twips,
    y: Twips,
    is_bezier_control: bool,
}

impl From<Point> for swf::Point<Twips> {
    fn from(point: Point) -> Self {
        Self::new(point.x, point.y)
    }
}

/// A continuous series of edges in a path.
/// Fill segments are directed, because the winding determines the fill-rule.
/// Stroke segments are undirected.
#[derive(Clone, Debug)]
struct PathSegment {
    pub points: Vec<Point>,
}

impl PathSegment {
    fn new(start: swf::Point<Twips>) -> Self {
        Self {
            points: vec![Point {
                x: start.x,
                y: start.y,
                is_bezier_control: false,
            }],
        }
    }

    fn reset(&mut self, start: swf::Point<Twips>) {
        self.points.clear();
        self.points.push(Point {
            x: start.x,
            y: start.y,
            is_bezier_control: false,
        });
    }

    /// Flips the direction of the path segment.
    /// Flash fill paths are dual-sided, with fill style 1 indicating the positive side
    /// and fill style 0 indicating the negative. We have to flip fill style 0 paths
    /// in order to link them to fill style 1 paths.
    fn flip(&mut self) {
        self.points.reverse();
    }

    /// Adds an edge to the end of the path segment.
    fn add_point(&mut self, point: Point) {
        self.points.push(point);
    }

    fn is_empty(&self) -> bool {
        self.points.len() <= 1
    }

    fn start(&self) -> Option<swf::Point<Twips>> {
        Some((*self.points.first()?).into())
    }

    fn end(&self) -> Option<swf::Point<Twips>> {
        Some((*self.points.last()?).into())
    }

    fn is_closed(&self) -> bool {
        self.start() == self.end()
    }

    fn to_draw_commands(&self) -> impl '_ + Iterator<Item = DrawCommand> {
        assert!(!self.is_empty());
        let mut i = self.points.iter();
        let first = i.next().expect("Points should not be empty");
        std::iter::once(DrawCommand::MoveTo((*first).into())).chain(std::iter::from_fn(move || {
            match i.next() {
                Some(
                    point @ Point {
                        is_bezier_control: false,
                        ..
                    },
                ) => Some(DrawCommand::LineTo((*point).into())),
                Some(
                    point @ Point {
                        is_bezier_control: true,
                        ..
                    },
                ) => {
                    let end = i.next().expect("Bezier without endpoint");
                    Some(DrawCommand::QuadraticCurveTo {
                        control: (*point).into(),
                        anchor: (*end).into(),
                    })
                }
                None => None,
            }
        }))
    }
}

/// The internal path structure used by ShapeConverter.
///
/// Each path is uniquely identified by its fill/stroke style. But Flash gives
/// the path edges as an "edge soup" -- they can arrive in an arbitrary order.
/// We have to link the edges together for each path. This structure contains
/// a list of path segment, and each time a path segment is added, it will try
/// to merge it with an existing segment.
#[derive(Clone, Debug)]
struct PendingPath {
    /// The list of path segments for this fill/stroke.
    /// For fills, this should turn into a list of closed paths when the shape is complete.
    /// Strokes may or may not be closed.
    segments: Vec<PathSegment>,
}

impl PendingPath {
    fn new() -> Self {
        Self { segments: vec![] }
    }

    /// Adds a path segment to the path, attempting to link it to existing segments.
    fn add_segment(&mut self, mut new_segment: PathSegment) {
        if !new_segment.is_empty() {
            // Try to link this segment onto existing segments with a matching endpoint.
            // Both the start and the end points of the new segment can be linked.
            let mut start_open = true;
            let mut end_open = true;
            let mut i = 0;
            while (start_open || end_open) && i < self.segments.len() {
                let other = &mut self.segments[i];
                if start_open && other.end() == new_segment.start() {
                    other.points.extend_from_slice(&new_segment.points[1..]);
                    new_segment = self.segments.swap_remove(i);
                    start_open = false;
                } else if end_open && new_segment.end() == other.start() {
                    std::mem::swap(&mut other.points, &mut new_segment.points);
                    other.points.extend_from_slice(&new_segment.points[1..]);
                    new_segment = self.segments.swap_remove(i);
                    end_open = false;
                } else {
                    i += 1;
                }
            }
            // The segment can't link to any further segments. Add it to list.
            self.segments.push(new_segment);
        }
    }

    fn push_path(&mut self, segment: PathSegment) {
        self.segments.push(segment);
    }

    fn to_draw_commands(&self) -> impl '_ + Iterator<Item = DrawCommand> {
        self.segments.iter().flat_map(PathSegment::to_draw_commands)
    }
}

#[derive(Clone, Debug)]
pub struct ActivePath {
    style_id: u32,
    segment: PathSegment,
}

impl ActivePath {
    fn new() -> Self {
        Self {
            style_id: 0,
            segment: PathSegment::new(swf::Point::ZERO),
        }
    }

    fn add_point(&mut self, point: Point) {
        self.segment.add_point(point)
    }

    fn flush_fill(&mut self, start: swf::Point<Twips>, pending: &mut [PendingPath], flip: bool) {
        if self.style_id > 0 && !self.segment.is_empty() {
            if flip {
                self.segment.flip();
            }
            pending[self.style_id as usize - 1].add_segment(self.segment.clone());
        }
        self.segment.reset(start);
    }

    fn flush_stroke(&mut self, start: swf::Point<Twips>, pending: &mut [PendingPath]) {
        if self.style_id > 0 && !self.segment.is_empty() {
            pending[self.style_id as usize - 1].push_path(self.segment.clone());
        }
        self.segment.reset(start);
    }
}

pub struct ShapeConverter<'a> {
    // SWF shape commands.
    iter: std::slice::Iter<'a, swf::ShapeRecord>,

    // Pen position.
    cursor: swf::Point<Twips>,

    // Fill styles and line styles.
    // These change from StyleChangeRecords, and a flush occurs when these change.
    fill_styles: &'a [swf::FillStyle],
    line_styles: &'a [swf::LineStyle],

    fill_style0: ActivePath,
    fill_style1: ActivePath,
    line_style: ActivePath,
    winding_rule: FillRule,

    // Paths. These get flushed for each new layer.
    fills: Vec<PendingPath>,
    strokes: Vec<PendingPath>,

    // Output.
    commands: Vec<DrawPath<'a>>,
}

impl<'a> ShapeConverter<'a> {
    const DEFAULT_CAPACITY: usize = 512;

    fn from_shape(shape: &'a swf::Shape) -> Self {
        ShapeConverter {
            iter: shape.shape.iter(),

            cursor: swf::Point::ZERO,

            fill_styles: &shape.styles.fill_styles,
            line_styles: &shape.styles.line_styles,

            fill_style0: ActivePath::new(),
            fill_style1: ActivePath::new(),
            line_style: ActivePath::new(),

            fills: vec![PendingPath::new(); shape.styles.fill_styles.len()],
            strokes: vec![PendingPath::new(); shape.styles.line_styles.len()],

            commands: Vec::with_capacity(Self::DEFAULT_CAPACITY),

            winding_rule: if shape.flags.contains(swf::ShapeFlag::NON_ZERO_WINDING_RULE) {
                FillRule::NonZero
            } else {
                FillRule::EvenOdd
            },
        }
    }

    fn into_commands(mut self) -> Vec<DrawPath<'a>> {
        // As u32 is okay because SWF has a max of 65536 fills (TODO: should be u16?)
        let mut num_fill_styles = self.fill_styles.len() as u32;
        let mut num_line_styles = self.line_styles.len() as u32;
        while let Some(record) = self.iter.next() {
            match record {
                ShapeRecord::StyleChange(style_change) => {
                    if let Some(move_to) = &style_change.move_to {
                        self.cursor = *move_to;
                        // We've lifted the pen, so we're starting a new path.
                        // Flush the previous path.
                        self.flush_paths();
                    }

                    if let Some(styles) = &style_change.new_styles {
                        // A new style list is also used to indicate a new drawing layer.
                        self.flush_layer();
                        self.fill_styles = &styles.fill_styles;
                        self.line_styles = &styles.line_styles;
                        self.fills
                            .resize_with(self.fill_styles.len(), PendingPath::new);
                        self.strokes
                            .resize_with(self.line_styles.len(), PendingPath::new);
                        num_fill_styles = self.fill_styles.len() as u32;
                        num_line_styles = self.line_styles.len() as u32;
                    }

                    if let Some(new_style_id) = style_change.fill_style_1 {
                        self.fill_style1
                            .flush_fill(self.cursor, &mut self.fills, false);
                        // Validate in case we index an invalid fill style.
                        // <= because fill ID 0 (no fill) is implicit, so the array is actually 1-based
                        self.fill_style1.style_id = if new_style_id <= num_fill_styles {
                            new_style_id
                        } else {
                            0
                        };
                    }

                    if let Some(new_style_id) = style_change.fill_style_0 {
                        self.fill_style0
                            .flush_fill(self.cursor, &mut self.fills, true);
                        self.fill_style0.style_id = if new_style_id <= num_fill_styles {
                            new_style_id
                        } else {
                            0
                        }
                    }

                    if let Some(new_style_id) = style_change.line_style {
                        self.line_style.flush_stroke(self.cursor, &mut self.strokes);
                        self.line_style.style_id = if new_style_id <= num_line_styles {
                            new_style_id
                        } else {
                            0
                        }
                    }
                }
                ShapeRecord::StraightEdge { delta } => {
                    self.cursor += *delta;
                    self.visit_point(false);
                }
                ShapeRecord::CurvedEdge {
                    control_delta,
                    anchor_delta,
                } => {
                    self.cursor += *control_delta;
                    self.visit_point(true);

                    self.cursor += *anchor_delta;
                    self.visit_point(false);
                }
            }
        }

        // Flush any open paths.
        self.flush_layer();
        self.commands
    }

    /// Adds a point to the current path for the active fills/strokes.
    fn visit_point(&mut self, is_bezier_control: bool) {
        let point = Point {
            x: self.cursor.x,
            y: self.cursor.y,
            is_bezier_control,
        };
        if self.fill_style1.style_id > 0 {
            self.fill_style1.add_point(point);
        }
        if self.fill_style0.style_id > 0 {
            self.fill_style0.add_point(point);
        }
        if self.line_style.style_id > 0 {
            self.line_style.add_point(point);
        }
    }

    /// When the pen jumps to a new position, we reset the active path.
    fn flush_paths(&mut self) {
        // Move the current paths to the active list.
        self.fill_style1
            .flush_fill(self.cursor, &mut self.fills, false);
        self.fill_style0
            .flush_fill(self.cursor, &mut self.fills, true);
        self.line_style.flush_stroke(self.cursor, &mut self.strokes);
    }

    /// When a new layer starts, all paths are flushed and turned into drawing commands.
    fn flush_layer(&mut self) {
        self.flush_paths();

        // Draw fills, and then strokes.
        // Paths are drawn in order of style id, not based on the order of the draw commands.
        for (i, path) in self.fills.iter_mut().enumerate() {
            // These invariants are checked above (any invalid/empty fill ID should not have been added).
            debug_assert!(i < self.fill_styles.len());
            if path.segments.is_empty() {
                continue;
            }
            let style = unsafe { self.fill_styles.get_unchecked(i) };
            self.commands.push(DrawPath::Fill {
                style,
                commands: path.to_draw_commands().collect(),
                winding_rule: self.winding_rule,
            });
            path.segments.clear();
        }

        // Strokes are drawn last because they always appear on top of fills in the same layer.
        // Because path segments can either be open or closed, we convert each stroke segment into
        // a separate draw command.
        for (i, path) in self.strokes.iter_mut().enumerate() {
            debug_assert!(i < self.line_styles.len());
            let style = unsafe { self.line_styles.get_unchecked(i) };
            for segment in &path.segments {
                if segment.is_empty() {
                    continue;
                }
                self.commands.push(DrawPath::Stroke {
                    style,
                    is_closed: segment.is_closed(),
                    commands: segment.to_draw_commands().collect(),
                });
            }
            path.segments.clear();
        }
    }
}

/* SHAPEFLAG HITTEST (point-in-contour)
 *
 * To determine whether a point is inside a shape, we shoot a ray on the +x axis and calculate a winding number based
 * on the edges that intersect with the ray.
 *
 * For each edge:
 *  if the edge cross the ray downward (+y), we add 1 to the winding number.
 *  if the edge cross the ray upward (-y), we add -1 to the winding number.
 *
 * We must also handle intersection with edge endpoints consistently to avoid double counting:
 *  the initial point of an edge is considered for upwards rays.
 *  the final point of an edge is considered for downward rays.
 *
 * For SWF shapes, edges with fillstyle1 use clockwise winding, and edges with fillstyle0 use CCW winding (flip them).
 * We ignore any edges with fills on both sides (interior edges).
 *
 * If the final winding number is odd, then the point is inside the shape (for default even-odd winding).
 *
 * For strokes, we calculate the distance to the line segment or curve and compare it to the stroke width.
 * Note that Flash renders with a minimum stroke width of 1px (20 twips) that we must account for.
 * TODO: We currently don't consider non-round endcaps or joins, or stroke scaling flags.
 */

/// Test whether the given point in object space is contained within the contour of the given shape.
/// local_matrix is used to calculate the proper stroke widths.
pub fn shape_hit_test(
    shape: &swf::Shape,
    test_point: swf::Point<Twips>,
    local_matrix: &Matrix,
) -> bool {
    let mut cursor = swf::Point::ZERO;
    let mut winding = 0;

    let mut has_fill_style0 = false;
    let mut has_fill_style1 = false;

    let min_width = stroke_minimum_width(local_matrix);
    let mut stroke_width = None;
    let mut line_styles = &shape.styles.line_styles;

    for record in &shape.shape {
        match record {
            swf::ShapeRecord::StyleChange(style_change) => {
                // New styles indicates a new layer;
                // Check if the point is within the current layer, then reset winding.
                if let Some(new_styles) = &style_change.new_styles {
                    if winding & 0b1 != 0 {
                        return true;
                    }
                    line_styles = &new_styles.line_styles;
                    winding = 0;
                }

                if let Some(move_to) = &style_change.move_to {
                    cursor = *move_to;
                }

                if let Some(i) = style_change.fill_style_0 {
                    has_fill_style0 = i > 0;
                }
                if let Some(i) = style_change.fill_style_1 {
                    has_fill_style1 = i > 0;
                }
                if let Some(i) = style_change.line_style {
                    stroke_width = if i > 0 {
                        // Flash renders strokes with a 1px minimum width.
                        if let Some(line_style) = line_styles.get(i as usize - 1) {
                            let width = line_style.width().get() as f64;
                            let scaled_width = 0.5 * width.max(min_width);
                            Some((scaled_width, scaled_width * scaled_width))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                }
            }
            swf::ShapeRecord::StraightEdge { delta } => {
                let end = cursor + *delta;

                // If this edge has a fill style on only one-side, check for a crossing.
                if has_fill_style1 {
                    if !has_fill_style0 {
                        winding += winding_number_line(test_point, cursor, end);
                    }
                } else if has_fill_style0 {
                    winding += winding_number_line(test_point, end, cursor);
                }

                if let Some(width) = stroke_width {
                    if hit_test_stroke(test_point, cursor, end, width) {
                        return true;
                    }
                }

                cursor = end;
            }
            swf::ShapeRecord::CurvedEdge {
                control_delta,
                anchor_delta,
            } => {
                let control = cursor + *control_delta;
                let anchor = control + *anchor_delta;

                // If this edge has a fill style on only one-side, check for a crossing.
                if has_fill_style1 {
                    if !has_fill_style0 {
                        winding += winding_number_curve(test_point, cursor, control, anchor);
                    }
                } else if has_fill_style0 {
                    winding += winding_number_curve(test_point, anchor, control, cursor);
                }

                if let Some(width) = stroke_width {
                    if hit_test_stroke_curve(test_point, cursor, control, anchor, width) {
                        return true;
                    }
                }

                cursor = anchor;
            }
        }
    }
    winding & 0b1 != 0
}

/// Test whether the given point is contained within the paths specified by the draw commands.
pub fn draw_command_fill_hit_test(commands: &[DrawCommand], test_point: swf::Point<Twips>) -> bool {
    let mut cursor = swf::Point::ZERO;
    let mut fill_start = swf::Point::ZERO;
    let mut winding = 0;

    // Draw command only contains a single fill, so don't have to worry about fill styles.
    for command in commands {
        match command {
            DrawCommand::MoveTo(move_to) => {
                cursor = *move_to;
                fill_start = *move_to;
            }
            DrawCommand::LineTo(line_to) => {
                winding += winding_number_line(test_point, cursor, *line_to);
                cursor = *line_to;
            }
            DrawCommand::QuadraticCurveTo { control, anchor } => {
                winding += winding_number_curve(test_point, cursor, *control, *anchor);
                cursor = *anchor;
            }
            DrawCommand::CubicCurveTo {
                control_a,
                control_b,
                anchor,
            } => {
                lyon_geom::CubicBezierSegment {
                    from: lyon_geom::Point::new(cursor.x.to_pixels(), cursor.y.to_pixels()),
                    ctrl1: lyon_geom::Point::new(control_a.x.to_pixels(), control_a.y.to_pixels()),
                    ctrl2: lyon_geom::Point::new(control_b.x.to_pixels(), control_b.y.to_pixels()),
                    to: lyon_geom::Point::new(anchor.x.to_pixels(), anchor.y.to_pixels()),
                }
                .for_each_quadratic_bezier(
                    CUBIC_CURVE_TOLERANCE,
                    &mut |quadratic_curve| {
                        winding += winding_number_curve(
                            test_point,
                            swf::Point::from_pixels(quadratic_curve.from.x, quadratic_curve.from.y),
                            swf::Point::from_pixels(quadratic_curve.ctrl.x, quadratic_curve.ctrl.y),
                            swf::Point::from_pixels(quadratic_curve.to.x, quadratic_curve.to.y),
                        );
                    },
                );
                cursor = *anchor;
            }
        }
    }
    if cursor != fill_start {
        // Close fill.
        winding += winding_number_line(test_point, cursor, fill_start);
    }

    winding & 0b1 != 0
}

/// Test whether the given point is contained within the strokes specified by the draw commands.
/// local_matrix is used to calculate the minimum stroke width.
pub fn draw_command_stroke_hit_test(
    commands: &[DrawCommand],
    stroke_width: Twips,
    test_point: swf::Point<Twips>,
    local_matrix: &Matrix,
) -> bool {
    let stroke_min_width = stroke_minimum_width(local_matrix);
    let stroke_width = 0.5 * f64::max(stroke_width.get().into(), stroke_min_width);
    let stroke_widths = (stroke_width, stroke_width * stroke_width);
    let mut cursor = swf::Point::ZERO;
    for command in commands {
        match command {
            DrawCommand::MoveTo(move_to) => {
                cursor = *move_to;
            }
            DrawCommand::LineTo(line_to) => {
                if hit_test_stroke(test_point, cursor, *line_to, stroke_widths) {
                    return true;
                }
                cursor = *line_to;
            }
            DrawCommand::QuadraticCurveTo { control, anchor } => {
                if hit_test_stroke_curve(test_point, cursor, *control, *anchor, stroke_widths) {
                    return true;
                }
                cursor = *anchor;
            }
            DrawCommand::CubicCurveTo {
                control_a,
                control_b,
                anchor,
            } => {
                let mut hit = false;
                lyon_geom::CubicBezierSegment {
                    from: lyon_geom::Point::new(cursor.x.to_pixels(), cursor.y.to_pixels()),
                    ctrl1: lyon_geom::Point::new(control_a.x.to_pixels(), control_a.y.to_pixels()),
                    ctrl2: lyon_geom::Point::new(control_b.x.to_pixels(), control_b.y.to_pixels()),
                    to: lyon_geom::Point::new(anchor.x.to_pixels(), anchor.y.to_pixels()),
                }
                .for_each_quadratic_bezier(
                    CUBIC_CURVE_TOLERANCE,
                    &mut |quadratic_curve| {
                        if !hit
                            && hit_test_stroke_curve(
                                test_point,
                                swf::Point::from_pixels(
                                    quadratic_curve.from.x,
                                    quadratic_curve.from.y,
                                ),
                                swf::Point::from_pixels(
                                    quadratic_curve.ctrl.x,
                                    quadratic_curve.ctrl.y,
                                ),
                                swf::Point::from_pixels(quadratic_curve.to.x, quadratic_curve.to.y),
                                stroke_widths,
                            )
                        {
                            hit = true;
                        }
                    },
                );
                cursor = *anchor;

                if hit {
                    return true;
                }
            }
        }
    }

    false
}

/// Given a matrix, calculates the scale for stroke widths.
/// TODO: Verify the actual behavior; I think it's more like the average between scaleX and scaleY.
/// Does not yet support vertical/horizontal stroke scaling flags.
/// This might be better to add as a method to Matrix.
fn stroke_minimum_width(matrix: &Matrix) -> f64 {
    let sx = (matrix.a * matrix.a + matrix.b * matrix.b).sqrt();
    let sy = (matrix.c * matrix.c + matrix.d * matrix.d).sqrt();
    let scale: f64 = sx.max(sy).into();
    20.0 * scale
}

/// Returns whether the given point is inside the stroked line segment.
/// `width_sq` should be the squared width of the stroke.
fn hit_test_stroke(
    test_point: swf::Point<Twips>,
    begin: swf::Point<Twips>,
    end: swf::Point<Twips>,
    (stroke_width, stroke_width_sq): (f64, f64),
) -> bool {
    let px = test_point.x.get() as f64;
    let py = test_point.y.get() as f64;
    let x0 = begin.x.get() as f64;
    let y0 = begin.y.get() as f64;
    let x1 = end.x.get() as f64;
    let y1 = end.y.get() as f64;

    // Early exit: out of bounds
    let x_min = x0.min(x1);
    let x_max = x0.max(x1);
    if px < x_min - stroke_width || px > x_max + stroke_width {
        return false;
    }
    let y_min = y0.min(y1);
    let y_max = y0.max(y1);
    if py < y_min - stroke_width || py > y_max + stroke_width {
        return false;
    }

    // AB is the segment from `begin` to `end` and P is `test_point`.
    //  P
    //   .
    //    .
    //     A----->B
    // If AP dot AB is <= 0.0, then PA is pointing away from AB, so A is the closest point.
    let abx = x1 - x0;
    let aby = y1 - y0;
    let apx = px - x0;
    let apy = py - y0;
    let dot_a = abx * apx + aby * apy;
    let dist = if dot_a <= 0.0 {
        apx * apx + apy * apy
    } else {
        // If BP dot AB is >= 0.0, then BP is pointing away from BA, so B is the closest point.
        let bpx = px - x1;
        let bpy = py - y1;
        let dot_b = abx * bpx + aby * bpy;
        if dot_b >= 0.0 {
            bpx * bpx + bpy * bpy
        } else {
            // Otherwise, the closest point will be within the interval of the segment.
            // Project the point onto the segment.
            let len = abx * abx + aby * aby;
            let ex = apx - dot_a * abx / len;
            let ey = apy - dot_a * aby / len;
            ex * ex + ey * ey
        }
    };

    dist <= stroke_width_sq
}

/// Returns whether the given point is inside the stroked bezier curve.
/// `width_sq` should be the squared width of the stroke.
fn hit_test_stroke_curve(
    test_point: swf::Point<Twips>,
    begin: swf::Point<Twips>,
    control: swf::Point<Twips>,
    anchor: swf::Point<Twips>,
    (stroke_width, stroke_width_sq): (f64, f64),
) -> bool {
    let px = test_point.x.get() as f64;
    let py = test_point.y.get() as f64;
    let x0 = begin.x.get() as f64;
    let y0 = begin.y.get() as f64;
    let x1 = control.x.get() as f64;
    let y1 = control.y.get() as f64;
    let x2 = anchor.x.get() as f64;
    let y2 = anchor.y.get() as f64;

    // Early exit: out of bounds
    // TODO: Since this involves an expensive cubic, probably wortwhile to calculate the tight bounds for the curve:
    // https://www.iquilezles.org/www/articles/bezierbbox/bezierbbox.htm
    let x_min = x0.min(x1).min(x2);
    let x_max = x0.max(x1).max(x2);
    if px < x_min - stroke_width || px > x_max + stroke_width {
        return false;
    }

    let y_min = y0.min(y1).min(y2);
    let y_max = y0.max(y1).max(y2);
    if py < y_min - stroke_width || py > y_max + stroke_width {
        return false;
    }

    // The closest point on the curve will be normal to the curve.
    // The tangent of a quadratic bezier:
    // C'(t) = -2 * (1-t) * P0 + 2 * (1-t) * P1 + 2*t*P2
    // Dot product to determine when we are perpendicular to the tangent.
    // (point - C(t)) . C'(t) = 0
    // The result is a cubic polynomial that we can solve for.
    // After solving this polynomial, we choose the t with [0, 1.0] that gives us the minimum distance
    // (also considering the endcaps).
    // via http://blog.gludion.com/2009/08/distance-to-quadratic-bezier-curve.html

    let ax = x1 - x0;
    let ay = y1 - y0;
    let bx = x2 - x1 - ax;
    let by = y2 - y1 - ay;
    let mx = x0 - px;
    let my = y0 - py;

    let a = bx * bx + by * by;
    let b = 3.0 * (ax * bx + ay * by);
    let c = 2.0 * (ax * ax + ay * ay) + (mx * bx + my * by);
    let d = mx * ax + my * ay;

    let distance_to_curve = |t| -> f64 {
        // Sample bezier at the given t and return distance to the point.
        let comp_t = 1.0 - t;
        let cx = comp_t * comp_t * x0 + 2.0 * comp_t * t * x1 + t * t * x2;
        let cy = comp_t * comp_t * y0 + 2.0 * comp_t * t * y1 + t * t * y2;
        let dx = cx - px;
        let dy = cy - py;
        dx * dx + dy * dy
    };

    // Test end-caps
    let mut dist = distance_to_curve(0.0);
    dist = dist.min(distance_to_curve(1.0));

    // Test roots.
    for t in solve_cubic(a, b, c, d) {
        if t >= 0.0 && t <= 1.0 {
            dist = dist.min(distance_to_curve(t));
        }
    }

    dist <= stroke_width_sq
}

/// Calculates the winding number for a line segment relative to the given point.
fn winding_number_line(
    test_point: swf::Point<Twips>,
    begin: swf::Point<Twips>,
    end: swf::Point<Twips>,
) -> i32 {
    let d0 = test_point - begin;
    let d1 = end - begin;

    // Adjust winding number if we are on the left side of the segment.
    // An upward segment (-y) increments the winding number (including the initial endpoint).
    // A downward segment (+y) decrements the winding number (including the final endpoint)
    // Perp-dot indicates which side of the segment the point is on.
    if begin.y < test_point.y {
        if end.y >= test_point.y
            && (d1.dx.get() as i64) * (d0.dy.get() as i64)
                > (d1.dy.get() as i64) * (d0.dx.get() as i64)
        {
            return 1;
        }
    } else if end.y < test_point.y
        && (d1.dx.get() as i64) * (d0.dy.get() as i64) < (d1.dy.get() as i64) * (d0.dx.get() as i64)
    {
        return -1;
    }

    0
}

/// Calculates the winding number for a bezier curve around the given point.
fn winding_number_curve(
    test_point: swf::Point<Twips>,
    begin: swf::Point<Twips>,
    control: swf::Point<Twips>,
    anchor: swf::Point<Twips>,
) -> i32 {
    // Intersect a ray on the +x axis with the quadratic bezier.
    //
    // Translate so the test point and ray is at the origin.
    // The ray-curve intersection is solving the quadratic:
    // y_0*(1-t)^2 + y_1*2*t*(1-t) + y_2*t^2 = 0

    // However, there are two issues:
    // 1) Solving the quadratic needs to be numerically robust, particularly near the endpoints 0.0 and 1.0, and as the curve is tangent to the ray.
    //    We use the "Citardauq" method for improved numerical stability.
    // 2) The convention for including/excluding endpoints needs to act similarly to lines, with the initial point included if the curve is "upward",
    //    and the final point included if the curve is pointing "downward". This is complicated by the fact that the curve could be tangent to the ray
    //    at the endpoint (this is still considered "upward" or "downward" depending on the slope at earlier t).
    //    We solve this by splitting the curve into y-monotonic subcurves. This is helpful because
    //    a) each subcurve will have 1 intersection with the ray
    //    b) if the subcurve surrounds the ray, we know it has an intersection without having to check if t is in [0, 1]
    //    c) we know the winding of the segment upward/downward based on which root it contains

    let d0 = begin - test_point;
    let d1 = control - test_point;
    let d2 = anchor - test_point;

    // Early exit: all control points out of bounds.
    if (d0.dy < Twips::ZERO && d1.dy < Twips::ZERO && d2.dy < Twips::ZERO)
        || (d0.dy > Twips::ZERO && d1.dy > Twips::ZERO && d2.dy > Twips::ZERO)
        || (d0.dx <= Twips::ZERO && d1.dx <= Twips::ZERO && d2.dx <= Twips::ZERO)
    {
        return 0;
    }

    let x0 = d0.dx.get() as f64;
    let y0 = d0.dy.get() as f64;
    let x1 = d1.dx.get() as f64;
    let y1 = d1.dy.get() as f64;
    let x2 = d2.dx.get() as f64;
    let y2 = d2.dy.get() as f64;

    let a = y0 - 2.0 * y1 + y2;
    let b = 2.0 * (y1 - y0);
    let c = y0;

    let (t0, t1) = solve_quadratic(a, b, c);
    let is_t0_valid = t0.is_finite();
    let is_t1_valid = t1.is_finite();
    if !is_t0_valid && !is_t1_valid {
        return 0;
    }

    // Split the curve into two y-monotonic segments.
    let mut winding = 0;
    let ax = x0 - 2.0 * x1 + x2;
    let bx = 2.0 * (x1 - x0);
    let t_extrema = -0.5 * b / a;
    let is_monotonic = t_extrema <= 0.0 || t_extrema >= 1.0;
    if a >= 0.0 {
        // Downward opening parabola.
        let y_min = if is_monotonic {
            y0.min(y2)
        } else {
            a * t_extrema * t_extrema + b * t_extrema + c
        };

        // First subcurve is moving upward, include initial point.
        if is_t0_valid && y0 >= 0.0 && y_min < 0.0 {
            // If curve point is to the right of the ray origin (x > 0), the ray will hit it.
            // We don't have to check 0 <= t <= 1 check because we've already guaranteed that the subcurve
            // straddles the ray.
            let x = x0 + bx * t0 + ax * t0 * t0;
            if x > 0.0 {
                winding += 1;
            }
        }

        // Second subcurve is moving downard, include final point.
        if is_t1_valid && y_min < 0.0 && y2 >= 0.0 {
            let x = x0 + bx * t1 + ax * t1 * t1;
            if x > 0.0 {
                winding -= 1;
            }
        }
    } else {
        // Upward opening parabola.
        let y_max = if is_monotonic {
            y0.max(y2)
        } else {
            a * t_extrema * t_extrema + b * t_extrema + c
        };

        // First subcurve is moving downward, include extrema point.
        if is_t1_valid && y0 < 0.0 && y_max >= 0.0 {
            let x = x0 + bx * t1 + ax * t1 * t1;
            if x > 0.0 {
                winding -= 1;
            }
        }

        // Second subcurve is moving upward, include extrema point.
        if is_t0_valid && y_max >= 0.0 && y2 < 0.0 {
            let x = x0 + bx * t0 + ax * t0 * t0;
            if x > 0.0 {
                winding += 1;
            }
        }
    }

    winding
}

const COEFFICIENT_EPSILON: f64 = 0.0000001;

/// Returns the roots of the quadratic ax^2 + bx + c = 0.
/// The roots may not be unique. NAN is returned for invalid roots. The first root will be where
/// the curve is sloping upward, the second root will be where the curve is slopping downward.
/// Uses the "Citardauq" formula for numerical stability.
/// See https://math.stackexchange.com/questions/866331
fn solve_quadratic(a: f64, b: f64, c: f64) -> (f64, f64) {
    if a.abs() <= COEFFICIENT_EPSILON {
        // Nearly linear, solve as linear equation.
        if b >= 0.0 {
            return (f64::NAN, -c / b);
        } else {
            return (-c / b, f64::NAN);
        }
    }
    let mut disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return (f64::NAN, f64::NAN);
    }
    disc = disc.sqrt();
    // Order the roots so that the first root is where the curve slopes upward,
    // and the second root is where the root slopes downward.
    if b >= 0.0 {
        let root0 = (-b - disc) / (2.0 * a);
        let root1 = c / (a * root0);
        (root0, root1)
    } else {
        let root0 = (-b + disc) / (2.0 * a);
        let root1 = c / (a * root0);
        (root1, root0)
    }
}

/// Returns the roots of a cubic polynomial, ax^3 + bx^2 + cx + d = 0
/// from http://www.cplusplus.com/forum/beginner/234717/
/// The roots are not necessarily unique.
/// TODO: This probably isn't numerically robust
#[allow(clippy::many_single_char_names)]
fn solve_cubic(a: f64, b: f64, c: f64, d: f64) -> SmallVec<[f64; 3]> {
    let mut roots = SmallVec::new();

    if a.abs() <= COEFFICIENT_EPSILON {
        // Fall back to quadratic formula.
        let (t0, t1) = solve_quadratic(b, c, d);
        #[allow(clippy::tuple_array_conversions)]
        roots.extend_from_slice(&[t0, t1]);
        return roots;
    }

    // Reduce to a "depressed cubic", x^3 + px + q = 0
    // https://en.wikipedia.org/wiki/Cubic_equation#Cardano's_formula
    let p = (b * b - 3.0 * a * c) / (9.0 * a * a);
    let q = (9.0 * a * b * c - 27.0 * a * a * d - 2.0 * b * b * b) / (54.0 * a * a * a);
    let offset = b / (3.0 * a);
    let disc = p * p * p - q * q;

    // The discriminant determines the number of real roots.
    if disc > 0.0 {
        let theta = f64::acos(q / (p * f64::sqrt(p)));
        let r = 2.0 * f64::sqrt(p);
        let t0 = r * f64::cos(theta / 3.0) - offset;
        let t1 = r * f64::cos((theta + 2.0 * std::f64::consts::PI) / 3.0) - offset;
        let t2 = r * f64::cos((theta + 4.0 * std::f64::consts::PI) / 3.0) - offset;
        roots.extend([t0, t1, t2]);
    } else {
        let gamma1 = f64::cbrt(q + f64::sqrt(-disc));
        let gamma2 = f64::cbrt(q - f64::sqrt(-disc));

        let t0 = gamma1 + gamma2 - offset;
        let t1 = -0.5 * (gamma1 + gamma2) - offset;
        roots.push(t0);
        if disc == 0.0 {
            roots.push(t1);
        }
    }

    roots
}

/// Converts an SWF glyph into an SWF shape, for ease of use by rendering backends.
pub fn swf_glyph_to_shape(glyph: &swf::Glyph) -> swf::Shape {
    // Per SWF19 p.164, the FontBoundsTable can contain empty bounds for every glyph (reserved).
    // SWF19 says this is true through SWFv7, but it seems like it might be generally true?
    // In any case, we have to be sure to calculate the shape bounds ourselves to make a proper
    // SVG.
    let bounds = glyph
        .bounds
        .clone()
        .filter(|b| b.x_min != b.x_max || b.y_min != b.y_max)
        .unwrap_or_else(|| calculate_shape_bounds(&glyph.shape_records));
    swf::Shape {
        version: 2,
        id: 0,
        shape_bounds: bounds.clone(),
        edge_bounds: bounds,
        flags: swf::ShapeFlag::HAS_SCALING_STROKES | swf::ShapeFlag::NON_ZERO_WINDING_RULE,
        styles: swf::ShapeStyles {
            fill_styles: vec![swf::FillStyle::Color(swf::Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            })],
            line_styles: vec![],
        },
        shape: glyph.shape_records.clone(),
    }
}

/// Scale mode used by strokes in a shape.
///
/// Determines how the line thickness is affected by the shape's transform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineScaleMode {
    None = 0,
    Horizontal,
    Vertical,
    Both,
}

/// Helper type for calculating line widths for a transformed shape.
pub struct LineScales<'a> {
    matrix: &'a Matrix,
    scales: Option<[f32; 4]>,
}

impl<'a> LineScales<'a> {
    /// Create a new line scaler for the given matrix.
    #[inline]
    pub fn new(matrix: &'a Matrix) -> Self {
        Self {
            matrix,
            scales: None,
        }
    }

    /// Returns the final width of a line after transformation.
    #[inline]
    pub fn transform_width(&mut self, width: f32, scale_mode: LineScaleMode) -> f32 {
        // Lazily calculate the scale to avoid doing so for shapes that have no strokes.
        let scales = self.scales.get_or_insert_with(|| {
            let line_scale_x = f32::abs(self.matrix.a + self.matrix.c);
            let line_scale_y = f32::abs(self.matrix.b + self.matrix.d);
            let line_scale =
                ((line_scale_x * line_scale_x + line_scale_y * line_scale_y) / 2.0).sqrt();
            [1.0, line_scale_x, line_scale_y, line_scale]
        });
        let scaled_width = width * scales[scale_mode as usize];
        // Flash draws all strokes with a minimum width of 1 pixel.
        // This usually occurs in "hairline" strokes (exported with width of 1 twip).
        scaled_width.max(1.0)
    }
}

pub fn quadratic_curve_bounds(
    start: swf::Point<Twips>,
    stroke_width: Twips,
    control: swf::Point<Twips>,
    anchor: swf::Point<Twips>,
) -> Rectangle<Twips> {
    // extremes
    let from_x = start.x.to_pixels();
    let from_y = start.y.to_pixels();
    let anchor_x = anchor.x.to_pixels();
    let anchor_y = anchor.y.to_pixels();
    let control_x = control.x.to_pixels();
    let control_y = control.y.to_pixels();

    let mut min_x = from_x.min(anchor_x);
    let mut min_y = from_y.min(anchor_y);
    let mut max_x = from_x.max(anchor_x);
    let mut max_y = from_y.max(anchor_y);

    if control_x < min_x || control_x > max_x {
        let t_x = ((from_x - control_x) / (from_x - (control_x * 2.0) + anchor_x)).clamp(0.0, 1.0);
        let s_x = 1.0 - t_x;
        let q_x = s_x * s_x * from_x + (s_x * 2.0) * t_x * control_x + t_x * t_x * anchor_x;

        min_x = min_x.min(q_x);
        max_x = max_x.max(q_x);
    }

    if control_y < min_y || control_y > max_y {
        let t_y = ((from_y - control_y) / (from_y - (control_y * 2.0) + anchor_y)).clamp(0.0, 1.0);
        let s_y = 1.0 - t_y;
        let q_y = s_y * s_y * from_y + (s_y * 2.0) * t_y * control_y + t_y * t_y * anchor_y;

        min_y = min_y.min(q_y);
        max_y = max_y.max(q_y);
    }

    let radius = stroke_width / 2;
    Rectangle::default()
        .encompass(swf::Point::new(
            Twips::from_pixels(min_x) - radius,
            Twips::from_pixels(min_y) - radius,
        ))
        .encompass(swf::Point::new(
            Twips::from_pixels(max_x) + radius,
            Twips::from_pixels(max_y) + radius,
        ))
}

pub fn cubic_curve_bounds(
    start: swf::Point<Twips>,
    stroke_width: Twips,
    control_a: swf::Point<Twips>,
    control_b: swf::Point<Twips>,
    anchor: swf::Point<Twips>,
) -> Rectangle<Twips> {
    // [NA] Should we just move most of our math in this file to lyon_geom?
    let bounds = lyon_geom::CubicBezierSegment {
        from: lyon_geom::Point::new(start.x.to_pixels(), start.y.to_pixels()),
        ctrl1: lyon_geom::Point::new(control_a.x.to_pixels(), control_a.y.to_pixels()),
        ctrl2: lyon_geom::Point::new(control_b.x.to_pixels(), control_b.y.to_pixels()),
        to: lyon_geom::Point::new(anchor.x.to_pixels(), anchor.y.to_pixels()),
    }
    .bounding_box();

    let radius = stroke_width / 2;
    Rectangle {
        x_min: Twips::from_pixels(bounds.min.x) - radius,
        x_max: Twips::from_pixels(bounds.max.x) + radius,
        y_min: Twips::from_pixels(bounds.min.y) - radius,
        y_max: Twips::from_pixels(bounds.max.y) + radius,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swf::PointDelta;

    const FILL_STYLES: [FillStyle; 1] = [FillStyle::Color(swf::Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    })];

    const LINE_STYLES: [LineStyle; 0] = [];

    /// Convenience method to quickly make a shape,
    fn build_shape(records: Vec<ShapeRecord>) -> swf::Shape {
        let bounds = calculate_shape_bounds(&records);
        swf::Shape {
            version: 2,
            id: 1,
            shape_bounds: bounds.clone(),
            edge_bounds: bounds,
            flags: swf::ShapeFlag::HAS_SCALING_STROKES,
            styles: swf::ShapeStyles {
                fill_styles: FILL_STYLES.to_vec(),
                line_styles: LINE_STYLES.to_vec(),
            },
            shape: records,
        }
    }

    /// A simple solid square.
    #[test]
    fn basic_shape() {
        let shape = build_shape(vec![
            ShapeRecord::StyleChange(Box::new(swf::StyleChangeData {
                move_to: Some(swf::Point::from_pixels(100.0, 100.0)),
                fill_style_0: None,
                fill_style_1: Some(1),
                line_style: None,
                new_styles: None,
            })),
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(100.0, 0.0),
            },
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(0.0, 100.0),
            },
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(-100.0, 0.0),
            },
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(0.0, -100.0),
            },
        ]);
        let commands = ShapeConverter::from_shape(&shape).into_commands();
        let expected = vec![DrawPath::Fill {
            style: &FILL_STYLES[0],
            commands: vec![
                DrawCommand::MoveTo(swf::Point::from_pixels(100.0, 100.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(200.0, 100.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(200.0, 200.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(100.0, 200.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(100.0, 100.0)),
            ],
            winding_rule: FillRule::EvenOdd,
        }];
        assert_eq!(commands, expected);
    }

    /// A solid square with one edge flipped (fillstyle0 instead of fillstyle1).
    #[test]
    fn flipped_edges() {
        let shape = build_shape(vec![
            ShapeRecord::StyleChange(Box::new(swf::StyleChangeData {
                move_to: Some(swf::Point::from_pixels(100.0, 100.0)),
                fill_style_0: None,
                fill_style_1: Some(1),
                line_style: None,
                new_styles: None,
            })),
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(100.0, 0.0),
            },
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(0.0, 100.0),
            },
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(-100.0, 0.0),
            },
            ShapeRecord::StyleChange(Box::new(swf::StyleChangeData {
                move_to: Some(swf::Point::from_pixels(100.0, 100.0)),
                fill_style_0: Some(1),
                fill_style_1: Some(0),
                line_style: None,
                new_styles: None,
            })),
            ShapeRecord::StraightEdge {
                delta: PointDelta::from_pixels(0.0, 100.0),
            },
        ]);
        let commands = ShapeConverter::from_shape(&shape).into_commands();
        let expected = vec![DrawPath::Fill {
            style: &FILL_STYLES[0],
            commands: vec![
                DrawCommand::MoveTo(swf::Point::from_pixels(100.0, 100.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(200.0, 100.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(200.0, 200.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(100.0, 200.0)),
                DrawCommand::LineTo(swf::Point::from_pixels(100.0, 100.0)),
            ],
            winding_rule: FillRule::EvenOdd,
        }];
        assert_eq!(commands, expected);
    }

    #[test]
    fn test_winding_number_line() {
        fn test(
            test_point: swf::Point<Twips>,
            begin: swf::Point<Twips>,
            end: swf::Point<Twips>,
            expected: i32,
        ) {
            let result = winding_number_line(test_point, begin, end);

            assert_eq!(
                expected, result,
                "result (winding number line) should match"
            );
        }

        // Test data taken from a real-world case:
        // https://github.com/ruffle-rs/ruffle/issues/11077
        // Overflow bugs can make this test case fail.
        test(
            swf::Point::new(Twips::new(0), Twips::new(-665)),
            swf::Point::new(Twips::new(44868), Twips::new(-41726)),
            swf::Point::new(Twips::new(44868), Twips::new(8275)),
            1,
        );
    }
}
