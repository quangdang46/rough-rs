use crate::core::{Op, OpSet, OpSetType, OpType, ResolvedOptions};
use crate::geometry::Point;
use crate::math::RngHelper;

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

    fn op_name(op: OpType) -> &'static str {
        match op {
            OpType::Move => "move",
            OpType::BCurveTo => "bcurveTo",
            OpType::LineTo => "lineTo",
        }
    }
}
