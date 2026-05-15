use crate::core::{Op, OpSet, OpSetType, OpType, ResolvedOptions};
use crate::geometry::Point;
use crate::math::RngHelper;
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

pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, options: &ResolvedOptions) -> OpSet {
    let mut rng = RngHelper::new(options.seed);
    OpSet::new(
        OpSetType::Path,
        double_line_ops(x1, y1, x2, y2, options, &mut rng, false),
    )
}

pub fn linear_path(points: &[Point], close: bool, options: &ResolvedOptions) -> OpSet {
    let len = points.len();
    if len > 2 {
        let mut rng = RngHelper::new(options.seed);
        let mut ops = Vec::new();
        for pair in points.windows(2) {
            ops.extend(double_line_ops(
                pair[0][0], pair[0][1], pair[1][0], pair[1][1], options, &mut rng, false,
            ));
        }
        if close {
            ops.extend(double_line_ops(
                points[len - 1][0],
                points[len - 1][1],
                points[0][0],
                points[0][1],
                options,
                &mut rng,
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
    let points = [
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y + height],
    ];
    polygon(&points, options)
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
