use crate::core::{FillStyle, Op, OpSet, OpSetType, OpType, ResolvedOptions};
use crate::geometry::Point;
use crate::math::{random_unit, RngHelper};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurveError {
    NotEnoughPoints,
}

impl fmt::Display for CurveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughPoints => f.write_str("a curve must have at least three points"),
        }
    }
}

impl std::error::Error for CurveError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EllipseParams {
    pub rx: f64,
    pub ry: f64,
    pub increment: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EllipseResult {
    pub opset: OpSet,
    pub estimated_points: Vec<Point>,
}

pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    OpSet::new(
        OpSetType::Path,
        double_line_ops(x1, y1, x2, y2, options, &mut rng, false),
    )
}

pub fn linear_path(points: &[Point], close: bool, options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    linear_path_with_rng(points, close, options, &mut rng)
}

pub fn linear_path_with_rng(
    points: &[Point],
    close: bool,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let len = points.len();
    if len > 2 {
        let mut ops = Vec::new();
        for pair in points.windows(2) {
            ops.extend(double_line_ops(
                pair[0][0], pair[0][1], pair[1][0], pair[1][1], options, rng, false,
            ));
        }
        if close {
            ops.extend(double_line_ops(
                points[len - 1][0],
                points[len - 1][1],
                points[0][0],
                points[0][1],
                options,
                rng,
                false,
            ));
        }
        OpSet::new(OpSetType::Path, ops)
    } else if len == 2 {
        line(
            points[0][0],
            points[0][1],
            points[1][0],
            points[1][1],
            options,
        )
    } else {
        empty_path(options)
    }
}

pub fn polygon(points: &[Point], options: &ResolvedOptions) -> OpSet {
    linear_path(points, true, options)
}

pub fn rectangle(x: f64, y: f64, width: f64, height: f64, options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    rectangle_with_rng(x, y, width, height, options, &mut rng)
}

pub fn rectangle_with_rng(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let points = [
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y + height],
    ];
    linear_path_with_rng(&points, true, options, rng)
}

pub fn ellipse(x: f64, y: f64, width: f64, height: f64, options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    let params = generate_ellipse_params(width, height, options, &mut rng);
    ellipse_with_params(x, y, options, params, &mut rng).opset
}

#[allow(clippy::too_many_arguments)]
pub fn arc(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    start: f64,
    stop: f64,
    closed: bool,
    rough_closure: bool,
    options: &ResolvedOptions,
) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    arc_with_rng(
        x,
        y,
        width,
        height,
        start,
        stop,
        closed,
        rough_closure,
        options,
        &mut rng,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn arc_with_rng(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    start: f64,
    stop: f64,
    closed: bool,
    rough_closure: bool,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let cx = x;
    let cy = y;
    let mut rx = (width / 2.0).abs();
    let mut ry = (height / 2.0).abs();
    rx += rng.offset_symmetric(rx * 0.01, options.roughness, 1.0);
    ry += rng.offset_symmetric(ry * 0.01, options.roughness, 1.0);
    let mut strt = start;
    let mut stp = stop;
    while strt < 0.0 {
        strt += std::f64::consts::PI * 2.0;
        stp += std::f64::consts::PI * 2.0;
    }
    if stp - strt > std::f64::consts::PI * 2.0 {
        strt = 0.0;
        stp = std::f64::consts::PI * 2.0;
    }
    let ellipse_inc = (std::f64::consts::PI * 2.0) / options.curve_step_count;
    let arc_inc = (ellipse_inc / 2.0).min((stp - strt) / 2.0);
    let mut ops = arc_ops(arc_inc, cx, cy, rx, ry, strt, stp, 1.0, options, rng);
    if !options.disable_multi_stroke {
        ops.extend(arc_ops(
            arc_inc, cx, cy, rx, ry, strt, stp, 1.5, options, rng,
        ));
    }
    if closed {
        if rough_closure {
            ops.extend(double_line_ops(
                cx,
                cy,
                cx + rx * strt.cos(),
                cy + ry * strt.sin(),
                options,
                rng,
                false,
            ));
            ops.extend(double_line_ops(
                cx,
                cy,
                cx + rx * stp.cos(),
                cy + ry * stp.sin(),
                options,
                rng,
                false,
            ));
        } else {
            ops.push(Op::new(OpType::LineTo, vec![cx, cy]));
            ops.push(Op::new(
                OpType::LineTo,
                vec![cx + rx * strt.cos(), cy + ry * strt.sin()],
            ));
        }
    }
    OpSet::new(OpSetType::Path, ops)
}

pub fn curve(points: &[Point], options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    curve_with_rng(points, options, &mut rng)
}

pub fn curve_with_rng(points: &[Point], options: &ResolvedOptions, rng: &mut RngHelper) -> OpSet {
    if points.is_empty() {
        return empty_path(options);
    }
    let mut ops = curve_with_offset(points, 1.0 * (1.0 + options.roughness * 0.2), options, rng);
    if !options.disable_multi_stroke {
        let mut overlay_rng = RngHelper::new(if options.seed != 0 {
            options.seed + 1
        } else {
            0
        });
        ops.extend(curve_with_offset(
            points,
            1.5 * (1.0 + options.roughness * 0.22),
            options,
            &mut overlay_rng,
        ));
    }
    OpSet::new(OpSetType::Path, ops)
}

#[cfg(feature = "svg_path")]
pub fn svg_path(path: &str, options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    svg_path_with_rng(path, options, &mut rng)
}

#[cfg(feature = "svg_path")]
pub fn svg_path_with_rng(path: &str, options: &ResolvedOptions, rng: &mut RngHelper) -> OpSet {
    let Ok(segments) = normalized_svg_segments(path) else {
        return empty_path(options);
    };
    let mut ops = Vec::new();
    let mut first = [0.0, 0.0];
    let mut current = [0.0, 0.0];
    for segment in segments {
        match segment {
            NormalizedSvgSegment::Move(point) => {
                current = point;
                first = point;
            }
            NormalizedSvgSegment::Line(point) => {
                ops.extend(double_line_ops(
                    current[0], current[1], point[0], point[1], options, rng, false,
                ));
                current = point;
            }
            NormalizedSvgSegment::Cubic([x1, y1, x2, y2, x, y]) => {
                ops.extend(bezier_to_ops(x1, y1, x2, y2, x, y, current, options, rng));
                current = [x, y];
            }
            NormalizedSvgSegment::Close => {
                ops.extend(double_line_ops(
                    current[0], current[1], first[0], first[1], options, rng, false,
                ));
                current = first;
            }
        }
    }
    OpSet::new(OpSetType::Path, ops)
}

#[cfg(feature = "svg_path")]
pub fn points_on_path(path: &str, tolerance: f64, distance: Option<f64>) -> Vec<Vec<Point>> {
    let Ok(segments) = normalized_svg_segments(path) else {
        return Vec::new();
    };
    let mut sets = Vec::new();
    let mut current_points = Vec::new();
    let mut start = [0.0, 0.0];
    let mut pending_curve = Vec::new();

    for segment in segments {
        match segment {
            NormalizedSvgSegment::Move(point) => {
                append_pending_path_points(
                    &mut sets,
                    &mut current_points,
                    &mut pending_curve,
                    tolerance,
                );
                start = point;
                current_points.push(point);
            }
            NormalizedSvgSegment::Line(point) => {
                append_pending_curve_points(&mut current_points, &mut pending_curve, tolerance);
                current_points.push(point);
            }
            NormalizedSvgSegment::Cubic([x1, y1, x2, y2, x, y]) => {
                if pending_curve.is_empty() {
                    let last_point = current_points.last().copied().unwrap_or(start);
                    pending_curve.push(last_point);
                }
                pending_curve.push([x1, y1]);
                pending_curve.push([x2, y2]);
                pending_curve.push([x, y]);
            }
            NormalizedSvgSegment::Close => {
                append_pending_curve_points(&mut current_points, &mut pending_curve, tolerance);
                current_points.push(start);
            }
        }
    }
    append_pending_path_points(
        &mut sets,
        &mut current_points,
        &mut pending_curve,
        tolerance,
    );

    if let Some(distance) = distance {
        if distance != 0.0 {
            return sets
                .into_iter()
                .filter_map(|set| {
                    let simplified = simplify(&set, distance);
                    (!simplified.is_empty()).then_some(simplified)
                })
                .collect();
        }
    }
    sets
}

pub fn generate_ellipse_params(
    width: f64,
    height: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> EllipseParams {
    let psq = (std::f64::consts::PI
        * 2.0
        * (((width / 2.0).powi(2) + (height / 2.0).powi(2)) / 2.0).sqrt())
    .sqrt();
    let step_count = options
        .curve_step_count
        .max((options.curve_step_count / 200.0_f64.sqrt()) * psq)
        .ceil();
    let increment = (std::f64::consts::PI * 2.0) / step_count;
    let curve_fit_randomness = 1.0 - options.curve_fitting;
    let mut rx = (width / 2.0).abs();
    let mut ry = (height / 2.0).abs();
    rx += rng.offset_symmetric(rx * curve_fit_randomness, options.roughness, 1.0);
    ry += rng.offset_symmetric(ry * curve_fit_randomness, options.roughness, 1.0);
    EllipseParams { rx, ry, increment }
}

pub fn ellipse_with_params(
    x: f64,
    y: f64,
    options: &ResolvedOptions,
    ellipse_params: EllipseParams,
    rng: &mut RngHelper,
) -> EllipseResult {
    let nested_max = rng.offset(0.4, 1.0, options.roughness, 1.0);
    let overlap = ellipse_params.increment * rng.offset(0.1, nested_max, options.roughness, 1.0);
    let (ap1, cp1) = compute_ellipse_points(
        ellipse_params.increment,
        x,
        y,
        ellipse_params.rx,
        ellipse_params.ry,
        1.0,
        overlap,
        options,
        rng,
    );
    let mut ops = curve_ops(&ap1, None, options, rng);
    if !options.disable_multi_stroke && options.roughness != 0.0 {
        let (ap2, _) = compute_ellipse_points(
            ellipse_params.increment,
            x,
            y,
            ellipse_params.rx,
            ellipse_params.ry,
            1.5,
            0.0,
            options,
            rng,
        );
        ops.extend(curve_ops(&ap2, None, options, rng));
    }

    EllipseResult {
        estimated_points: cp1,
        opset: OpSet::new(OpSetType::Path, ops),
    }
}

pub fn solid_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let mut ops = Vec::new();
    for points in polygon_list {
        if points.len() > 2 {
            let offset = options.max_randomness_offset;
            ops.push(Op::new(
                OpType::Move,
                vec![
                    points[0][0] + rng.offset_symmetric(offset, options.roughness, 1.0),
                    points[0][1] + rng.offset_symmetric(offset, options.roughness, 1.0),
                ],
            ));
            for point in &points[1..] {
                ops.push(Op::new(
                    OpType::LineTo,
                    vec![
                        point[0] + rng.offset_symmetric(offset, options.roughness, 1.0),
                        point[1] + rng.offset_symmetric(offset, options.roughness, 1.0),
                    ],
                ));
            }
        }
    }
    OpSet::new(OpSetType::FillPath, ops)
}

pub fn pattern_fill_polygons(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    match options.fill_style {
        FillStyle::Solid => solid_fill_polygon(polygon_list, options, rng),
        FillStyle::CrossHatch => cross_hatch_fill_polygon(polygon_list, options, rng),
        FillStyle::Dots => dots_fill_polygon(polygon_list, options, rng),
        FillStyle::Dashed => dashed_fill_polygon(polygon_list, options, rng),
        FillStyle::Zigzag => zigzag_fill_polygon(polygon_list, options, rng),
        FillStyle::ZigzagLine => zigzag_line_fill_polygon(polygon_list, options, rng),
        _ => hachure_fill_polygon(polygon_list, options, rng),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn pattern_fill_arc(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    start: f64,
    stop: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let cx = x;
    let cy = y;
    let mut rx = (width / 2.0).abs();
    let mut ry = (height / 2.0).abs();
    rx += rng.offset_symmetric(rx * 0.01, options.roughness, 1.0);
    ry += rng.offset_symmetric(ry * 0.01, options.roughness, 1.0);
    let mut strt = start;
    let mut stp = stop;
    while strt < 0.0 {
        strt += std::f64::consts::PI * 2.0;
        stp += std::f64::consts::PI * 2.0;
    }
    if stp - strt > std::f64::consts::PI * 2.0 {
        strt = 0.0;
        stp = std::f64::consts::PI * 2.0;
    }
    let increment = (stp - strt) / options.curve_step_count;
    let mut points = Vec::new();
    let mut angle = strt;
    while angle <= stp {
        points.push([cx + rx * angle.cos(), cy + ry * angle.sin()]);
        angle += increment;
    }
    points.push([cx + rx * stp.cos(), cy + ry * stp.sin()]);
    points.push([cx, cy]);
    pattern_fill_polygons(&[points], options, rng)
}

pub fn hachure_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let lines = polygon_hachure_lines(polygon_list, options, rng);
    OpSet::new(
        OpSetType::FillSketch,
        render_fill_lines(&lines, options, rng),
    )
}

pub fn cross_hatch_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let first = polygon_hachure_lines(polygon_list, options, rng);
    let mut ops = render_fill_lines(&first, options, rng);
    let mut cross_options = options.clone();
    cross_options.hachure_angle = options.hachure_angle + 90.0;
    let second = polygon_hachure_lines(polygon_list, &cross_options, rng);
    ops.extend(render_fill_lines(&second, &cross_options, rng));
    OpSet::new(OpSetType::FillSketch, ops)
}

pub fn dots_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let mut dot_options = options.clone();
    dot_options.hachure_angle = 0.0;
    let lines = polygon_hachure_lines(polygon_list, &dot_options, rng);
    let gap = dot_options.effective_hachure_gap().max(0.1);
    let fill_weight = dot_options.effective_fill_weight();
    let random_offset = gap / 4.0;
    let mut ops = Vec::new();
    for line in lines {
        let length = distance(line[0], line[1]);
        let count = (length / gap).ceil() as usize;
        let count = count.saturating_sub(1);
        let offset = length - (count as f64 * gap);
        let x = ((line[0][0] + line[1][0]) / 2.0) - (gap / 4.0);
        let min_y = line[0][1].min(line[1][1]);
        for i in 0..count {
            let y = min_y + offset + (i as f64 * gap);
            let cx = (x - random_offset) + random_unit() * 2.0 * random_offset;
            let cy = (y - random_offset) + random_unit() * 2.0 * random_offset;
            let params = generate_ellipse_params(fill_weight, fill_weight, &dot_options, rng);
            ops.extend(
                ellipse_with_params(cx, cy, &dot_options, params, rng)
                    .opset
                    .ops,
            );
        }
    }
    OpSet::new(OpSetType::FillSketch, ops)
}

pub fn dashed_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let lines = polygon_hachure_lines(polygon_list, options, rng);
    let offset = options.effective_dash_offset();
    let gap = options.effective_dash_gap();
    let period = offset + gap;
    if period <= 0.0 {
        return OpSet::new(OpSetType::FillSketch, Vec::new());
    }

    let mut ops = Vec::new();
    for line in lines {
        let length = distance(line[0], line[1]);
        let count = (length / period).floor() as usize;
        let start_offset = (length + gap - (count as f64 * period)) / 2.0;
        let (p1, p2) = ordered_by_x(line);
        let alpha = ((p2[1] - p1[1]) / (p2[0] - p1[0])).atan();
        let cos = alpha.cos();
        let sin = alpha.sin();
        for i in 0..count {
            let line_start = i as f64 * period;
            let line_end = line_start + offset;
            let start = [
                p1[0] + (line_start * cos) + (start_offset * cos),
                p1[1] + (line_start * sin) + (start_offset * sin),
            ];
            let end = [
                p1[0] + (line_end * cos) + (start_offset * cos),
                p1[1] + (line_end * sin) + (start_offset * sin),
            ];
            ops.extend(double_line_ops(
                start[0], start[1], end[0], end[1], options, rng, true,
            ));
        }
    }
    OpSet::new(OpSetType::FillSketch, ops)
}

pub fn zigzag_line_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let gap = options.effective_hachure_gap();
    let zigzag_offset = options.effective_zigzag_offset();
    if zigzag_offset <= 0.0 {
        return OpSet::new(OpSetType::FillSketch, Vec::new());
    }

    let mut zigzag_options = options.clone();
    zigzag_options.hachure_gap = gap + zigzag_offset;
    let lines = polygon_hachure_lines(polygon_list, &zigzag_options, rng);
    let diagonal = (2.0 * zigzag_offset.powi(2)).sqrt();
    let mut ops = Vec::new();
    for line in lines {
        let length = distance(line[0], line[1]);
        let count = (length / (2.0 * zigzag_offset)).round() as usize;
        let (p1, p2) = ordered_by_x(line);
        let alpha = ((p2[1] - p1[1]) / (p2[0] - p1[0])).atan();
        let cos = alpha.cos();
        let sin = alpha.sin();
        for i in 0..count {
            let line_start = i as f64 * 2.0 * zigzag_offset;
            let line_end = (i + 1) as f64 * 2.0 * zigzag_offset;
            let start = [p1[0] + (line_start * cos), p1[1] + (line_start * sin)];
            let end = [p1[0] + (line_end * cos), p1[1] + (line_end * sin)];
            let middle = [
                start[0] + diagonal * (alpha + std::f64::consts::FRAC_PI_4).cos(),
                start[1] + diagonal * (alpha + std::f64::consts::FRAC_PI_4).sin(),
            ];
            ops.extend(double_line_ops(
                start[0], start[1], middle[0], middle[1], options, rng, true,
            ));
            ops.extend(double_line_ops(
                middle[0], middle[1], end[0], end[1], options, rng, true,
            ));
        }
    }
    OpSet::new(OpSetType::FillSketch, ops)
}

pub fn zigzag_fill_polygon(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> OpSet {
    let gap = options.effective_hachure_gap().max(0.1);
    let mut hachure_options = options.clone();
    hachure_options.hachure_gap = gap;
    let lines = polygon_hachure_lines(polygon_list, &hachure_options, rng);
    let zigzag_angle = options.hachure_angle.to_radians();
    let delta_x = gap * 0.5 * zigzag_angle.cos();
    let delta_y = gap * 0.5 * zigzag_angle.sin();
    let mut zigzag_lines = Vec::new();
    for [p1, p2] in lines {
        if distance(p1, p2) != 0.0 {
            zigzag_lines.push([[p1[0] - delta_x, p1[1] + delta_y], p2]);
            zigzag_lines.push([[p1[0] + delta_x, p1[1] - delta_y], p2]);
        }
    }
    OpSet::new(
        OpSetType::FillSketch,
        render_fill_lines(&zigzag_lines, options, rng),
    )
}

pub fn polygon_hachure_lines(
    polygon_list: &[Vec<Point>],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> Vec<[Point; 2]> {
    let angle = options.hachure_angle + 90.0;
    let mut gap = options.hachure_gap;
    if gap < 0.0 {
        gap = options.stroke_width * 4.0;
    }
    gap = js_round(gap.max(0.1));
    let mut skip_offset = 1.0;
    if options.roughness >= 1.0 && rng.next_f64() > 0.7 {
        skip_offset = gap;
    }
    hachure_lines(polygon_list, gap, angle, skip_offset)
}

fn render_fill_lines(
    lines: &[[Point; 2]],
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> Vec<Op> {
    let mut ops = Vec::new();
    for line in lines {
        ops.extend(double_line_ops(
            line[0][0], line[0][1], line[1][0], line[1][1], options, rng, true,
        ));
    }
    ops
}

fn ordered_by_x(line: [Point; 2]) -> (Point, Point) {
    if line[0][0] > line[1][0] {
        (line[1], line[0])
    } else {
        (line[0], line[1])
    }
}

pub fn double_line_ops(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
    filling: bool,
) -> Vec<Op> {
    let single_stroke = if filling {
        options.disable_multi_stroke_fill
    } else {
        options.disable_multi_stroke
    };
    let mut ops = line_ops([x1, y1], [x2, y2], options, rng, true, false);
    if !single_stroke {
        ops.extend(line_ops([x1, y1], [x2, y2], options, rng, true, true));
    }
    ops
}

pub fn empty_path(_options: &ResolvedOptions) -> OpSet {
    OpSet::new(OpSetType::Path, Vec::new())
}

pub fn curve_to_bezier(
    points_in: &[Point],
    curve_tightness: f64,
) -> Result<Vec<Point>, CurveError> {
    let len = points_in.len();
    if len < 3 {
        return Err(CurveError::NotEnoughPoints);
    }

    let mut out = Vec::new();
    if len == 3 {
        out.push(points_in[0]);
        out.push(points_in[1]);
        out.push(points_in[2]);
        out.push(points_in[2]);
    } else {
        let mut points = Vec::with_capacity(points_in.len() + 2);
        points.push(points_in[0]);
        points.push(points_in[0]);
        for (index, point) in points_in.iter().enumerate().skip(1) {
            points.push(*point);
            if index == points_in.len() - 1 {
                points.push(*point);
            }
        }

        let s = 1.0 - curve_tightness;
        out.push(points[0]);
        for i in 1..points.len() - 2 {
            let current = points[i];
            let b1 = [
                current[0] + (s * points[i + 1][0] - s * points[i - 1][0]) / 6.0,
                current[1] + (s * points[i + 1][1] - s * points[i - 1][1]) / 6.0,
            ];
            let b2 = [
                points[i + 1][0] + (s * points[i][0] - s * points[i + 2][0]) / 6.0,
                points[i + 1][1] + (s * points[i][1] - s * points[i + 2][1]) / 6.0,
            ];
            out.push(b1);
            out.push(b2);
            out.push(points[i + 1]);
        }
    }

    Ok(out)
}

pub fn points_on_bezier_curves(
    points: &[Point],
    tolerance: f64,
    distance: Option<f64>,
) -> Vec<Point> {
    let mut new_points = Vec::new();
    let num_segments = points.len().saturating_sub(1) / 3;
    for i in 0..num_segments {
        get_points_on_bezier_curve_with_splitting(points, i * 3, tolerance, &mut new_points);
    }
    if let Some(distance) = distance {
        if distance > 0.0 {
            return simplify_points(&new_points, 0, new_points.len(), distance, &mut Vec::new());
        }
    }
    new_points
}

pub fn simplify(points: &[Point], distance: f64) -> Vec<Point> {
    simplify_points(points, 0, points.len(), distance, &mut Vec::new())
}

fn line_ops(
    start: Point,
    end: Point,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
    move_first: bool,
    overlay: bool,
) -> Vec<Op> {
    let [x1, y1] = start;
    let [x2, y2] = end;
    let length_sq = (x1 - x2).powi(2) + (y1 - y2).powi(2);
    let length = length_sq.sqrt();
    let roughness_gain = if length < 200.0 {
        1.0
    } else if length > 500.0 {
        0.4
    } else {
        -0.0016668 * length + 1.233334
    };

    let mut offset = options.max_randomness_offset;
    if offset * offset * 100.0 > length_sq {
        offset = length / 10.0;
    }
    let half_offset = offset / 2.0;
    let diverge_point = 0.2 + rng.next_f64() * 0.2;
    let mut mid_disp_x = options.bowing * options.max_randomness_offset * (y2 - y1) / 200.0;
    let mut mid_disp_y = options.bowing * options.max_randomness_offset * (x1 - x2) / 200.0;
    mid_disp_x = rng.offset_symmetric(mid_disp_x, options.roughness, roughness_gain);
    mid_disp_y = rng.offset_symmetric(mid_disp_y, options.roughness, roughness_gain);

    let preserve_vertices = options.preserve_vertices;
    let mut ops = Vec::with_capacity(2);
    if move_first {
        let start_offset = if overlay { half_offset } else { offset };
        ops.push(Op::new(
            OpType::Move,
            vec![
                x1 + if preserve_vertices {
                    0.0
                } else {
                    rng.offset_symmetric(start_offset, options.roughness, roughness_gain)
                },
                y1 + if preserve_vertices {
                    0.0
                } else {
                    rng.offset_symmetric(start_offset, options.roughness, roughness_gain)
                },
            ],
        ));
    }

    let curve_offset = if overlay { half_offset } else { offset };
    ops.push(Op::new(
        OpType::BCurveTo,
        vec![
            mid_disp_x
                + x1
                + (x2 - x1) * diverge_point
                + rng.offset_symmetric(curve_offset, options.roughness, roughness_gain),
            mid_disp_y
                + y1
                + (y2 - y1) * diverge_point
                + rng.offset_symmetric(curve_offset, options.roughness, roughness_gain),
            mid_disp_x
                + x1
                + 2.0 * (x2 - x1) * diverge_point
                + rng.offset_symmetric(curve_offset, options.roughness, roughness_gain),
            mid_disp_y
                + y1
                + 2.0 * (y2 - y1) * diverge_point
                + rng.offset_symmetric(curve_offset, options.roughness, roughness_gain),
            x2 + if preserve_vertices {
                0.0
            } else {
                rng.offset_symmetric(curve_offset, options.roughness, roughness_gain)
            },
            y2 + if preserve_vertices {
                0.0
            } else {
                rng.offset_symmetric(curve_offset, options.roughness, roughness_gain)
            },
        ],
    ));

    ops
}

fn curve_ops(
    points: &[Point],
    close_point: Option<Point>,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> Vec<Op> {
    let len = points.len();
    let mut ops = Vec::new();
    if len > 3 {
        let s = 1.0 - options.curve_tightness;
        ops.push(Op::new(OpType::Move, vec![points[1][0], points[1][1]]));
        for i in 1..len - 2 {
            let current = points[i];
            let b1 = [
                current[0] + (s * points[i + 1][0] - s * points[i - 1][0]) / 6.0,
                current[1] + (s * points[i + 1][1] - s * points[i - 1][1]) / 6.0,
            ];
            let b2 = [
                points[i + 1][0] + (s * points[i][0] - s * points[i + 2][0]) / 6.0,
                points[i + 1][1] + (s * points[i][1] - s * points[i + 2][1]) / 6.0,
            ];
            let b3 = points[i + 1];
            ops.push(Op::new(
                OpType::BCurveTo,
                vec![b1[0], b1[1], b2[0], b2[1], b3[0], b3[1]],
            ));
        }
        if let Some(close_point) = close_point {
            let ro = options.max_randomness_offset;
            ops.push(Op::new(
                OpType::LineTo,
                vec![
                    close_point[0] + rng.offset_symmetric(ro, options.roughness, 1.0),
                    close_point[1] + rng.offset_symmetric(ro, options.roughness, 1.0),
                ],
            ));
        }
    } else if len == 3 {
        ops.push(Op::new(OpType::Move, vec![points[1][0], points[1][1]]));
        ops.push(Op::new(
            OpType::BCurveTo,
            vec![
                points[1][0],
                points[1][1],
                points[2][0],
                points[2][1],
                points[2][0],
                points[2][1],
            ],
        ));
    } else if len == 2 {
        ops.extend(line_ops(points[0], points[1], options, rng, true, true));
    }
    ops
}

#[allow(clippy::too_many_arguments)]
fn arc_ops(
    increment: f64,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    strt: f64,
    stp: f64,
    offset: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> Vec<Op> {
    let rad_offset = strt + rng.offset_symmetric(0.1, options.roughness, 1.0);
    let mut points = Vec::new();
    points.push([
        rng.offset_symmetric(offset, options.roughness, 1.0)
            + cx
            + 0.9 * rx * (rad_offset - increment).cos(),
        rng.offset_symmetric(offset, options.roughness, 1.0)
            + cy
            + 0.9 * ry * (rad_offset - increment).sin(),
    ]);
    let mut angle = rad_offset;
    while angle <= stp {
        points.push([
            rng.offset_symmetric(offset, options.roughness, 1.0) + cx + rx * angle.cos(),
            rng.offset_symmetric(offset, options.roughness, 1.0) + cy + ry * angle.sin(),
        ]);
        angle += increment;
    }
    points.push([cx + rx * stp.cos(), cy + ry * stp.sin()]);
    points.push([cx + rx * stp.cos(), cy + ry * stp.sin()]);
    curve_ops(&points, None, options, rng)
}

fn curve_with_offset(
    points: &[Point],
    offset: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> Vec<Op> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut ps = Vec::with_capacity(points.len() + 2);
    ps.push([
        points[0][0] + rng.offset_symmetric(offset, options.roughness, 1.0),
        points[0][1] + rng.offset_symmetric(offset, options.roughness, 1.0),
    ]);
    ps.push([
        points[0][0] + rng.offset_symmetric(offset, options.roughness, 1.0),
        points[0][1] + rng.offset_symmetric(offset, options.roughness, 1.0),
    ]);
    for (index, point) in points.iter().enumerate().skip(1) {
        ps.push([
            point[0] + rng.offset_symmetric(offset, options.roughness, 1.0),
            point[1] + rng.offset_symmetric(offset, options.roughness, 1.0),
        ]);
        if index == points.len() - 1 {
            ps.push([
                point[0] + rng.offset_symmetric(offset, options.roughness, 1.0),
                point[1] + rng.offset_symmetric(offset, options.roughness, 1.0),
            ]);
        }
    }
    curve_ops(&ps, None, options, rng)
}

#[cfg(feature = "svg_path")]
#[derive(Debug, Clone, Copy, PartialEq)]
enum NormalizedSvgSegment {
    Move(Point),
    Line(Point),
    Cubic([f64; 6]),
    Close,
}

#[cfg(feature = "svg_path")]
fn normalized_svg_segments(path: &str) -> Result<Vec<NormalizedSvgSegment>, svgtypes::Error> {
    use svgtypes::{PathParser, PathSegment};

    let mut out = Vec::new();
    let mut current = [0.0, 0.0];
    let mut subpath = [0.0, 0.0];
    let mut last_type = '\0';
    let mut last_cubic_control = [0.0, 0.0];
    let mut last_quadratic_control = [0.0, 0.0];

    for segment in PathParser::from(path) {
        match segment? {
            PathSegment::MoveTo { abs, x, y } => {
                current = absolute_point(current, [x, y], abs);
                subpath = current;
                out.push(NormalizedSvgSegment::Move(current));
                last_type = 'M';
            }
            PathSegment::LineTo { abs, x, y } => {
                current = absolute_point(current, [x, y], abs);
                out.push(NormalizedSvgSegment::Line(current));
                last_type = 'L';
            }
            PathSegment::HorizontalLineTo { abs, x } => {
                current[0] = if abs { x } else { current[0] + x };
                out.push(NormalizedSvgSegment::Line(current));
                last_type = 'H';
            }
            PathSegment::VerticalLineTo { abs, y } => {
                current[1] = if abs { y } else { current[1] + y };
                out.push(NormalizedSvgSegment::Line(current));
                last_type = 'V';
            }
            PathSegment::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let c1 = absolute_point(current, [x1, y1], abs);
                let c2 = absolute_point(current, [x2, y2], abs);
                current = absolute_point(current, [x, y], abs);
                out.push(NormalizedSvgSegment::Cubic([
                    c1[0], c1[1], c2[0], c2[1], current[0], current[1],
                ]));
                last_cubic_control = c2;
                last_type = 'C';
            }
            PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                let c1 = if matches!(last_type, 'C' | 'S') {
                    [
                        current[0] + (current[0] - last_cubic_control[0]),
                        current[1] + (current[1] - last_cubic_control[1]),
                    ]
                } else {
                    current
                };
                let c2 = absolute_point(current, [x2, y2], abs);
                current = absolute_point(current, [x, y], abs);
                out.push(NormalizedSvgSegment::Cubic([
                    c1[0], c1[1], c2[0], c2[1], current[0], current[1],
                ]));
                last_cubic_control = c2;
                last_type = 'S';
            }
            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                let q = absolute_point(current, [x1, y1], abs);
                let end = absolute_point(current, [x, y], abs);
                push_quadratic_as_cubic(&mut out, current, q, end);
                current = end;
                last_quadratic_control = q;
                last_type = 'Q';
            }
            PathSegment::SmoothQuadratic { abs, x, y } => {
                let q = if matches!(last_type, 'Q' | 'T') {
                    [
                        current[0] + (current[0] - last_quadratic_control[0]),
                        current[1] + (current[1] - last_quadratic_control[1]),
                    ]
                } else {
                    current
                };
                let end = absolute_point(current, [x, y], abs);
                push_quadratic_as_cubic(&mut out, current, q, end);
                current = end;
                last_quadratic_control = q;
                last_type = 'T';
            }
            PathSegment::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                let end = absolute_point(current, [x, y], abs);
                let rx = rx.abs();
                let ry = ry.abs();
                if rx == 0.0 || ry == 0.0 {
                    out.push(NormalizedSvgSegment::Cubic([
                        current[0], current[1], end[0], end[1], end[0], end[1],
                    ]));
                } else if current != end {
                    for curve in
                        arc_to_cubic_curves(current, end, rx, ry, x_axis_rotation, large_arc, sweep)
                    {
                        out.push(NormalizedSvgSegment::Cubic(curve));
                    }
                }
                current = end;
                last_type = 'A';
            }
            PathSegment::ClosePath { .. } => {
                out.push(NormalizedSvgSegment::Close);
                current = subpath;
                last_type = 'Z';
            }
        }
    }

    Ok(out)
}

#[cfg(feature = "svg_path")]
fn absolute_point(current: Point, point: Point, abs: bool) -> Point {
    if abs {
        point
    } else {
        [current[0] + point[0], current[1] + point[1]]
    }
}

#[cfg(feature = "svg_path")]
fn push_quadratic_as_cubic(
    out: &mut Vec<NormalizedSvgSegment>,
    current: Point,
    control: Point,
    end: Point,
) {
    let c1 = [
        current[0] + 2.0 * (control[0] - current[0]) / 3.0,
        current[1] + 2.0 * (control[1] - current[1]) / 3.0,
    ];
    let c2 = [
        end[0] + 2.0 * (control[0] - end[0]) / 3.0,
        end[1] + 2.0 * (control[1] - end[1]) / 3.0,
    ];
    out.push(NormalizedSvgSegment::Cubic([
        c1[0], c1[1], c2[0], c2[1], end[0], end[1],
    ]));
}

#[cfg(feature = "svg_path")]
fn append_pending_curve_points(
    current_points: &mut Vec<Point>,
    pending_curve: &mut Vec<Point>,
    tolerance: f64,
) {
    if pending_curve.len() >= 4 {
        current_points.extend(points_on_bezier_curves(pending_curve, tolerance, None));
    }
    pending_curve.clear();
}

#[cfg(feature = "svg_path")]
fn append_pending_path_points(
    sets: &mut Vec<Vec<Point>>,
    current_points: &mut Vec<Point>,
    pending_curve: &mut Vec<Point>,
    tolerance: f64,
) {
    append_pending_curve_points(current_points, pending_curve, tolerance);
    if !current_points.is_empty() {
        sets.push(std::mem::take(current_points));
    }
}

#[cfg(feature = "svg_path")]
#[allow(clippy::too_many_arguments)]
fn bezier_to_ops(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x: f64,
    y: f64,
    current: Point,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> Vec<Op> {
    let random_offsets = [
        if options.max_randomness_offset != 0.0 {
            options.max_randomness_offset
        } else {
            1.0
        },
        if options.max_randomness_offset != 0.0 {
            options.max_randomness_offset
        } else {
            1.0
        } + 0.3,
    ];
    let iterations = if options.disable_multi_stroke { 1 } else { 2 };
    let preserve_vertices = options.preserve_vertices;
    let mut ops = Vec::new();
    for (i, offset) in random_offsets.iter().enumerate().take(iterations) {
        if i == 0 {
            ops.push(Op::new(OpType::Move, vec![current[0], current[1]]));
        } else {
            ops.push(Op::new(
                OpType::Move,
                vec![
                    current[0]
                        + if preserve_vertices {
                            0.0
                        } else {
                            rng.offset_symmetric(random_offsets[0], options.roughness, 1.0)
                        },
                    current[1]
                        + if preserve_vertices {
                            0.0
                        } else {
                            rng.offset_symmetric(random_offsets[0], options.roughness, 1.0)
                        },
                ],
            ));
        }
        let end = if preserve_vertices {
            [x, y]
        } else {
            [
                x + rng.offset_symmetric(*offset, options.roughness, 1.0),
                y + rng.offset_symmetric(*offset, options.roughness, 1.0),
            ]
        };
        ops.push(Op::new(
            OpType::BCurveTo,
            vec![
                x1 + rng.offset_symmetric(*offset, options.roughness, 1.0),
                y1 + rng.offset_symmetric(*offset, options.roughness, 1.0),
                x2 + rng.offset_symmetric(*offset, options.roughness, 1.0),
                y2 + rng.offset_symmetric(*offset, options.roughness, 1.0),
                end[0],
                end[1],
            ],
        ));
    }
    ops
}

#[cfg(feature = "svg_path")]
#[allow(clippy::too_many_arguments)]
fn arc_to_cubic_curves(
    start: Point,
    end: Point,
    mut rx: f64,
    mut ry: f64,
    angle: f64,
    large_arc_flag: bool,
    sweep_flag: bool,
) -> Vec<[f64; 6]> {
    let angle_rad = angle.to_radians();
    let (x1, y1) = rotate(start[0], start[1], -angle_rad);
    let (x2, y2) = rotate(end[0], end[1], -angle_rad);
    let x = (x1 - x2) / 2.0;
    let y = (y1 - y2) / 2.0;
    let mut h = (x * x) / (rx * rx) + (y * y) / (ry * ry);
    if h > 1.0 {
        h = h.sqrt();
        rx *= h;
        ry *= h;
    }
    let sign = if large_arc_flag == sweep_flag {
        -1.0
    } else {
        1.0
    };
    let rx_pow = rx * rx;
    let ry_pow = ry * ry;
    let left = rx_pow * ry_pow - rx_pow * y * y - ry_pow * x * x;
    let right = rx_pow * y * y + ry_pow * x * x;
    let k = sign * (left / right).abs().sqrt();
    let cx = k * rx * y / ry + (x1 + x2) / 2.0;
    let cy = k * -ry * x / rx + (y1 + y2) / 2.0;
    let mut f1 = js_round_to_digits((y1 - cy) / ry, 9).asin();
    let mut f2 = js_round_to_digits((y2 - cy) / ry, 9).asin();
    if x1 < cx {
        f1 = std::f64::consts::PI - f1;
    }
    if x2 < cx {
        f2 = std::f64::consts::PI - f2;
    }
    if f1 < 0.0 {
        f1 += std::f64::consts::PI * 2.0;
    }
    if f2 < 0.0 {
        f2 += std::f64::consts::PI * 2.0;
    }
    if sweep_flag && f1 > f2 {
        f1 -= std::f64::consts::PI * 2.0;
    }
    if !sweep_flag && f2 > f1 {
        f2 -= std::f64::consts::PI * 2.0;
    }

    let mut curves = Vec::new();
    arc_to_cubic_curves_recursive(
        &mut curves,
        [x1, y1],
        [x2, y2],
        rx,
        ry,
        sweep_flag,
        f1,
        f2,
        cx,
        cy,
    );
    for curve in &mut curves {
        let c1 = rotate(curve[0], curve[1], angle_rad);
        let c2 = rotate(curve[2], curve[3], angle_rad);
        let e = rotate(curve[4], curve[5], angle_rad);
        *curve = [c1.0, c1.1, c2.0, c2.1, e.0, e.1];
    }
    curves
}

#[cfg(feature = "svg_path")]
#[allow(clippy::too_many_arguments)]
fn arc_to_cubic_curves_recursive(
    out: &mut Vec<[f64; 6]>,
    start: Point,
    mut end: Point,
    rx: f64,
    ry: f64,
    sweep_flag: bool,
    f1: f64,
    f2: f64,
    cx: f64,
    cy: f64,
) {
    let mut segment_f2 = f2;
    let mut tail = None;
    let max = std::f64::consts::PI * 120.0 / 180.0;
    if (segment_f2 - f1).abs() > max {
        let f2_old = segment_f2;
        let end_old = end;
        segment_f2 = if sweep_flag && segment_f2 > f1 {
            f1 + max
        } else {
            f1 - max
        };
        end = [cx + rx * segment_f2.cos(), cy + ry * segment_f2.sin()];
        tail = Some((end, end_old, segment_f2, f2_old));
    }

    let df = segment_f2 - f1;
    let c1 = f1.cos();
    let s1 = f1.sin();
    let c2 = segment_f2.cos();
    let s2 = segment_f2.sin();
    let t = (df / 4.0).tan();
    let hx = 4.0 / 3.0 * rx * t;
    let hy = 4.0 / 3.0 * ry * t;
    let m1 = start;
    let mut m2 = [start[0] + hx * s1, start[1] - hy * c1];
    let m3 = [end[0] + hx * s2, end[1] - hy * c2];
    let m4 = end;
    m2[0] = 2.0 * m1[0] - m2[0];
    m2[1] = 2.0 * m1[1] - m2[1];
    out.push([m2[0], m2[1], m3[0], m3[1], m4[0], m4[1]]);

    if let Some((tail_start, tail_end, tail_f1, tail_f2)) = tail {
        arc_to_cubic_curves_recursive(
            out, tail_start, tail_end, rx, ry, sweep_flag, tail_f1, tail_f2, cx, cy,
        );
    }
}

#[cfg(feature = "svg_path")]
fn rotate(x: f64, y: f64, angle_rad: f64) -> (f64, f64) {
    (
        x * angle_rad.cos() - y * angle_rad.sin(),
        x * angle_rad.sin() + y * angle_rad.cos(),
    )
}

#[cfg(feature = "svg_path")]
fn js_round_to_digits(value: f64, digits: i32) -> f64 {
    let factor = 10_f64.powi(digits);
    (value * factor).round() / factor
}

#[allow(clippy::too_many_arguments)]
fn compute_ellipse_points(
    mut increment: f64,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    offset: f64,
    overlap: f64,
    options: &ResolvedOptions,
    rng: &mut RngHelper,
) -> (Vec<Point>, Vec<Point>) {
    let core_only = options.roughness == 0.0;
    let mut core_points = Vec::new();
    let mut all_points = Vec::new();

    if core_only {
        increment /= 4.0;
        all_points.push([cx + rx * (-increment).cos(), cy + ry * (-increment).sin()]);
        let mut angle = 0.0;
        while angle <= std::f64::consts::PI * 2.0 {
            let point = [cx + rx * angle.cos(), cy + ry * angle.sin()];
            core_points.push(point);
            all_points.push(point);
            angle += increment;
        }
        all_points.push([cx + rx, cy]);
        all_points.push([cx + rx * increment.cos(), cy + ry * increment.sin()]);
    } else {
        let rad_offset =
            rng.offset_symmetric(0.5, options.roughness, 1.0) - std::f64::consts::FRAC_PI_2;
        all_points.push([
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cx
                + 0.9 * rx * (rad_offset - increment).cos(),
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cy
                + 0.9 * ry * (rad_offset - increment).sin(),
        ]);
        let end_angle = std::f64::consts::PI * 2.0 + rad_offset - 0.01;
        let mut angle = rad_offset;
        while angle < end_angle {
            let point = [
                rng.offset_symmetric(offset, options.roughness, 1.0) + cx + rx * angle.cos(),
                rng.offset_symmetric(offset, options.roughness, 1.0) + cy + ry * angle.sin(),
            ];
            core_points.push(point);
            all_points.push(point);
            angle += increment;
        }
        all_points.push([
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cx
                + rx * (rad_offset + std::f64::consts::PI * 2.0 + overlap * 0.5).cos(),
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cy
                + ry * (rad_offset + std::f64::consts::PI * 2.0 + overlap * 0.5).sin(),
        ]);
        all_points.push([
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cx
                + 0.98 * rx * (rad_offset + overlap).cos(),
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cy
                + 0.98 * ry * (rad_offset + overlap).sin(),
        ]);
        all_points.push([
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cx
                + 0.9 * rx * (rad_offset + overlap * 0.5).cos(),
            rng.offset_symmetric(offset, options.roughness, 1.0)
                + cy
                + 0.9 * ry * (rad_offset + overlap * 0.5).sin(),
        ]);
    }

    (all_points, core_points)
}

fn distance(p1: Point, p2: Point) -> f64 {
    distance_sq(p1, p2).sqrt()
}

fn distance_sq(p1: Point, p2: Point) -> f64 {
    (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2)
}

fn distance_to_segment_sq(p: Point, v: Point, w: Point) -> f64 {
    let length_sq = distance_sq(v, w);
    if length_sq == 0.0 {
        return distance_sq(p, v);
    }
    let t = (((p[0] - v[0]) * (w[0] - v[0]) + (p[1] - v[1]) * (w[1] - v[1])) / length_sq)
        .clamp(0.0, 1.0);
    distance_sq(p, lerp(v, w, t))
}

fn lerp(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn flatness(points: &[Point], offset: usize) -> f64 {
    let p1 = points[offset];
    let p2 = points[offset + 1];
    let p3 = points[offset + 2];
    let p4 = points[offset + 3];
    let mut ux = 3.0 * p2[0] - 2.0 * p1[0] - p4[0];
    ux *= ux;
    let mut uy = 3.0 * p2[1] - 2.0 * p1[1] - p4[1];
    uy *= uy;
    let mut vx = 3.0 * p3[0] - 2.0 * p4[0] - p1[0];
    vx *= vx;
    let mut vy = 3.0 * p3[1] - 2.0 * p4[1] - p1[1];
    vy *= vy;
    if ux < vx {
        ux = vx;
    }
    if uy < vy {
        uy = vy;
    }
    ux + uy
}

fn get_points_on_bezier_curve_with_splitting(
    points: &[Point],
    offset: usize,
    tolerance: f64,
    out_points: &mut Vec<Point>,
) {
    if flatness(points, offset) < tolerance {
        let p0 = points[offset];
        if let Some(last) = out_points.last() {
            if distance(*last, p0) > 1.0 {
                out_points.push(p0);
            }
        } else {
            out_points.push(p0);
        }
        out_points.push(points[offset + 3]);
    } else {
        let p1 = points[offset];
        let p2 = points[offset + 1];
        let p3 = points[offset + 2];
        let p4 = points[offset + 3];
        let q1 = lerp(p1, p2, 0.5);
        let q2 = lerp(p2, p3, 0.5);
        let q3 = lerp(p3, p4, 0.5);
        let r1 = lerp(q1, q2, 0.5);
        let r2 = lerp(q2, q3, 0.5);
        let red = lerp(r1, r2, 0.5);
        get_points_on_bezier_curve_with_splitting(&[p1, q1, r1, red], 0, tolerance, out_points);
        get_points_on_bezier_curve_with_splitting(&[red, r2, q3, p4], 0, tolerance, out_points);
    }
}

fn simplify_points(
    points: &[Point],
    start: usize,
    end: usize,
    epsilon: f64,
    out_points: &mut Vec<Point>,
) -> Vec<Point> {
    if end <= start || points.is_empty() {
        return out_points.clone();
    }

    let s = points[start];
    let e = points[end - 1];
    let mut max_dist_sq = 0.0;
    let mut max_index = start + 1;
    for (i, point) in points
        .iter()
        .enumerate()
        .take(end.saturating_sub(1))
        .skip(start + 1)
    {
        let dist_sq = distance_to_segment_sq(*point, s, e);
        if dist_sq > max_dist_sq {
            max_dist_sq = dist_sq;
            max_index = i;
        }
    }

    if max_dist_sq.sqrt() > epsilon {
        simplify_points(points, start, max_index + 1, epsilon, out_points);
        simplify_points(points, max_index, end, epsilon, out_points);
    } else {
        if out_points.is_empty() {
            out_points.push(s);
        }
        out_points.push(e);
    }

    out_points.clone()
}

fn hachure_lines(
    polygon_list: &[Vec<Point>],
    hachure_gap: f64,
    hachure_angle: f64,
    hachure_step_offset: f64,
) -> Vec<[Point; 2]> {
    let gap = hachure_gap.max(0.1);
    let mut polygons = polygon_list.to_vec();
    let rotation_center = [0.0, 0.0];
    if hachure_angle != 0.0 {
        rotate_polygons_degrees(&mut polygons, rotation_center, hachure_angle);
    }
    let mut lines = straight_hachure_lines(&polygons, gap, hachure_step_offset);
    if hachure_angle != 0.0 {
        rotate_lines_degrees(&mut lines, rotation_center, -hachure_angle);
    }
    lines
}

#[derive(Debug, Clone, Copy)]
struct HachureEdge {
    ymin: f64,
    ymax: f64,
    x: f64,
    islope: f64,
}

#[derive(Debug, Clone, Copy)]
struct ActiveHachureEdge {
    edge: HachureEdge,
}

fn straight_hachure_lines(
    polygons: &[Vec<Point>],
    mut gap: f64,
    hachure_step_offset: f64,
) -> Vec<[Point; 2]> {
    let mut vertex_array = Vec::new();
    for polygon in polygons {
        if polygon.is_empty() {
            continue;
        }
        let mut vertices = polygon.clone();
        if vertices.first() != vertices.last() {
            vertices.push(vertices[0]);
        }
        if vertices.len() > 2 {
            vertex_array.push(vertices);
        }
    }

    let mut lines = Vec::new();
    gap = gap.max(0.1);
    let mut edges = Vec::new();
    for vertices in &vertex_array {
        for pair in vertices.windows(2) {
            let p1 = pair[0];
            let p2 = pair[1];
            if p1[1] != p2[1] {
                let ymin = p1[1].min(p2[1]);
                edges.push(HachureEdge {
                    ymin,
                    ymax: p1[1].max(p2[1]),
                    x: if ymin == p1[1] { p1[0] } else { p2[0] },
                    islope: (p2[0] - p1[0]) / (p2[1] - p1[1]),
                });
            }
        }
    }
    edges.sort_by(compare_edges);
    if edges.is_empty() {
        return lines;
    }

    let mut active_edges: Vec<ActiveHachureEdge> = Vec::new();
    let mut y = edges[0].ymin;
    let mut iteration = 0.0;
    while !active_edges.is_empty() || !edges.is_empty() {
        if !edges.is_empty() {
            let mut ix = None;
            for (i, edge) in edges.iter().enumerate() {
                if edge.ymin > y {
                    break;
                }
                ix = Some(i);
            }
            if let Some(ix) = ix {
                let removed = edges.drain(0..=ix).collect::<Vec<_>>();
                active_edges.extend(removed.into_iter().map(|edge| ActiveHachureEdge { edge }));
            }
        }

        active_edges.retain(|active| active.edge.ymax > y);
        active_edges.sort_by(|a, b| compare_f64(a.edge.x, b.edge.x));

        if (hachure_step_offset != 1.0 || js_remainder(iteration, gap) == 0.0)
            && active_edges.len() > 1
        {
            let mut i = 0;
            while i + 1 < active_edges.len() {
                let ce = active_edges[i].edge;
                let ne = active_edges[i + 1].edge;
                lines.push([[js_round(ce.x), y], [js_round(ne.x), y]]);
                i += 2;
            }
        }

        y += hachure_step_offset;
        for active in &mut active_edges {
            active.edge.x += hachure_step_offset * active.edge.islope;
        }
        iteration += 1.0;
    }

    lines
}

fn rotate_polygons_degrees(polygons: &mut [Vec<Point>], center: Point, degrees: f64) {
    for polygon in polygons {
        for point in polygon {
            *point = rotate_point_degrees(*point, center, degrees);
        }
    }
}

fn rotate_lines_degrees(lines: &mut [[Point; 2]], center: Point, degrees: f64) {
    for line in lines {
        line[0] = rotate_point_degrees(line[0], center, degrees);
        line[1] = rotate_point_degrees(line[1], center, degrees);
    }
}

fn rotate_point_degrees(point: Point, center: Point, degrees: f64) -> Point {
    let angle = (std::f64::consts::PI / 180.0) * degrees;
    let cos = angle.cos();
    let sin = angle.sin();
    [
        ((point[0] - center[0]) * cos) - ((point[1] - center[1]) * sin) + center[0],
        ((point[0] - center[0]) * sin) + ((point[1] - center[1]) * cos) + center[1],
    ]
}

fn compare_edges(a: &HachureEdge, b: &HachureEdge) -> std::cmp::Ordering {
    compare_f64(a.ymin, b.ymin)
        .then(compare_f64(a.x, b.x))
        .then(compare_f64(a.ymax, b.ymax))
}

fn compare_f64(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

fn js_round(value: f64) -> f64 {
    (value + 0.5).floor()
}

fn js_remainder(a: f64, b: f64) -> f64 {
    a - (a / b).trunc() * b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Options;
    use approx::assert_relative_eq;
    use serde_json::Value;

    #[test]
    fn line_uses_cubic_beziers_and_double_stroke_by_default() {
        let opset = line(10.0, 10.0, 20.0, 20.0, &ResolvedOptions::default());

        assert_eq!(opset.ops.len(), 4);
        assert_eq!(opset.ops[0].op, OpType::Move);
        assert_eq!(opset.ops[1].op, OpType::BCurveTo);
        assert_eq!(opset.ops[2].op, OpType::Move);
        assert_eq!(opset.ops[3].op, OpType::BCurveTo);
    }

    #[test]
    fn line_honors_disable_multi_stroke() {
        let options = ResolvedOptions::from_options(&Options {
            disable_multi_stroke: Some(true),
            ..Options::default()
        });

        let opset = line(10.0, 10.0, 20.0, 20.0, &options);

        assert_eq!(opset.ops.len(), 2);
        assert_eq!(opset.ops[0].op, OpType::Move);
        assert_eq!(opset.ops[1].op, OpType::BCurveTo);
    }

    #[test]
    fn line_preserves_vertices_when_requested() {
        let options = ResolvedOptions::from_options(&Options {
            preserve_vertices: Some(true),
            seed: Some(42),
            ..Options::default()
        });

        let opset = line(10.0, 10.0, 20.0, 20.0, &options);

        assert_eq!(opset.ops[0].data[0], 10.0);
        assert_eq!(opset.ops[0].data[1], 10.0);
        assert_eq!(opset.ops[1].data[4], 20.0);
        assert_eq!(opset.ops[1].data[5], 20.0);
    }

    #[test]
    fn seeded_line_matches_legacy_fixture_ops() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference.json")).unwrap();
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == "line_seed_1")
            .expect("line fixture should exist");
        let expected_ops = case["drawable"]["sets"][0]["ops"].as_array().unwrap();
        let options = ResolvedOptions::from_options(&Options {
            seed: Some(1),
            ..Options::default()
        });

        let actual = line(10.0, 10.0, 200.0, 100.0, &options);

        assert_eq!(actual.ops.len(), expected_ops.len());
        for (actual, expected) in actual.ops.iter().zip(expected_ops) {
            assert_eq!(op_name(actual.op), expected["op"].as_str().unwrap());
            let expected_data = expected["data"].as_array().unwrap();
            assert_eq!(actual.data.len(), expected_data.len());
            for (actual_value, expected_value) in actual.data.iter().zip(expected_data) {
                assert_relative_eq!(
                    *actual_value,
                    expected_value.as_f64().unwrap(),
                    epsilon = 1e-12
                );
            }
        }
    }

    #[test]
    fn seeded_rectangle_matches_legacy_fixture_ops() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference.json")).unwrap();
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == "rectangle_seed_42")
            .expect("rectangle fixture should exist");
        let expected_ops = case["drawable"]["sets"][0]["ops"].as_array().unwrap();
        let options = ResolvedOptions::from_options(&Options {
            seed: Some(42),
            ..Options::default()
        });

        let actual = rectangle(10.0, 10.0, 200.0, 100.0, &options);

        assert_eq!(actual.ops.len(), expected_ops.len());
        for (actual, expected) in actual.ops.iter().zip(expected_ops) {
            assert_eq!(op_name(actual.op), expected["op"].as_str().unwrap());
            let expected_data = expected["data"].as_array().unwrap();
            assert_eq!(actual.data.len(), expected_data.len());
            for (actual_value, expected_value) in actual.data.iter().zip(expected_data) {
                assert_relative_eq!(
                    *actual_value,
                    expected_value.as_f64().unwrap(),
                    epsilon = 1e-12
                );
            }
        }
    }

    #[test]
    fn seeded_ellipse_matches_legacy_fixture_ops() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference.json")).unwrap();
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == "ellipse_seed_99")
            .expect("ellipse fixture should exist");
        let expected_ops = case["drawable"]["sets"][0]["ops"].as_array().unwrap();
        let options = ResolvedOptions::from_options(&Options {
            seed: Some(99),
            ..Options::default()
        });

        let actual = ellipse(100.0, 100.0, 200.0, 150.0, &options);

        assert_eq!(actual.ops.len(), expected_ops.len());
        for (actual, expected) in actual.ops.iter().zip(expected_ops) {
            assert_eq!(op_name(actual.op), expected["op"].as_str().unwrap());
            let expected_data = expected["data"].as_array().unwrap();
            assert_eq!(actual.data.len(), expected_data.len());
            for (actual_value, expected_value) in actual.data.iter().zip(expected_data) {
                assert_relative_eq!(
                    *actual_value,
                    expected_value.as_f64().unwrap(),
                    epsilon = 1e-10
                );
            }
        }
    }

    #[test]
    fn ellipse_params_scale_step_count_with_size() {
        let options = ResolvedOptions::from_options(&Options {
            seed: Some(42),
            ..Options::default()
        });
        let mut small_rng = RngHelper::new(options.seed);
        let mut large_rng = RngHelper::new(options.seed);
        let small = generate_ellipse_params(20.0, 20.0, &options, &mut small_rng);
        let large = generate_ellipse_params(400.0, 300.0, &options, &mut large_rng);

        assert!(large.increment < small.increment);
    }

    #[test]
    fn line_handles_degenerate_input_without_invalid_numbers() {
        let options = ResolvedOptions::from_options(&Options {
            seed: Some(1),
            ..Options::default()
        });
        let opset = line(10.0, 10.0, 10.0, 10.0, &options);

        for op in opset.ops {
            for value in op.data {
                assert!(value.is_finite());
            }
        }
    }

    #[test]
    fn curve_to_bezier_matches_points_on_curve_shape() {
        let fixture: Value =
            serde_json::from_str(include_str!("../tests/fixtures/reference.json")).unwrap();
        let points = [[0.0, 0.0], [10.0, 15.0], [20.0, 0.0], [30.0, 10.0]];
        let bezier = curve_to_bezier(&points, 0.0).unwrap();
        let expected = fixture["curveUtilities"]["curveToBezier"]
            .as_array()
            .expect("curve utility fixture should exist");

        assert_eq!(bezier.len(), expected.len());
        for (actual, expected) in bezier.iter().zip(expected) {
            assert_relative_eq!(actual[0], expected[0].as_f64().unwrap(), epsilon = 1e-12);
            assert_relative_eq!(actual[1], expected[1].as_f64().unwrap(), epsilon = 1e-12);
        }
    }

    #[test]
    fn curve_to_bezier_rejects_short_inputs() {
        assert_eq!(
            curve_to_bezier(&[[0.0, 0.0], [1.0, 1.0]], 0.0),
            Err(CurveError::NotEnoughPoints)
        );
    }

    #[test]
    fn points_on_bezier_curves_flattens_and_simplifies() {
        let bezier = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [20.0, 10.0]];
        let points = points_on_bezier_curves(&bezier, 0.15, None);
        let simplified = points_on_bezier_curves(&bezier, 0.15, Some(5.0));

        assert!(points.len() > 2);
        assert_eq!(points.first(), Some(&[0.0, 0.0]));
        assert_eq!(points.last(), Some(&[20.0, 10.0]));
        assert!(simplified.len() <= points.len());
    }

    #[test]
    fn simplify_keeps_endpoints() {
        let points = [[0.0, 0.0], [1.0, 0.01], [2.0, 0.0], [3.0, 1.0]];
        let simplified = simplify(&points, 0.1);

        assert_eq!(simplified.first(), Some(&[0.0, 0.0]));
        assert_eq!(simplified.last(), Some(&[3.0, 1.0]));
    }

    fn op_name(op: OpType) -> &'static str {
        match op {
            OpType::Move => "move",
            OpType::BCurveTo => "bcurveTo",
            OpType::LineTo => "lineTo",
        }
    }
}
