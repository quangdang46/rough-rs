use approx::assert_relative_eq;
use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, FillStyle, Generator, Op, OpSetType, OpType, Options, ShapeType};
use serde_json::Value;
use std::error::Error;
use std::io;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn polygon_and_linear_path_use_distinct_shape_and_fill_behavior() {
    let generator = Generator::new(Config::default());
    let points = [[10.0, 10.0], [140.0, 20.0], [120.0, 90.0], [30.0, 110.0]];

    let linear = generator.linear_path(
        &points,
        Some(Options {
            seed: Some(42),
            fill: Some("red".to_string()),
            ..Options::default()
        }),
    );
    let polygon = generator.polygon(
        &points,
        Some(Options {
            seed: Some(42),
            fill: Some("red".to_string()),
            fill_style: Some(FillStyle::Solid),
            ..Options::default()
        }),
    );

    assert_eq!(linear.shape, ShapeType::LinearPath);
    assert_eq!(linear.sets.len(), 1);
    assert_eq!(polygon.shape, ShapeType::Polygon);
    assert_eq!(polygon.sets.len(), 2);
    assert_eq!(drawable_to_paths(&polygon)[0].fill, "red");
}

#[test]
fn primitive_paths_match_roughjs_fixture_strings_where_float_formatting_allows() -> TestResult {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json"))?;
    let generator = Generator::new(Config::default());

    let linear = generator.linear_path(
        &[[10.0, 10.0], [40.0, 70.0], [100.0, 30.0], [160.0, 90.0]],
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );
    assert_eq!(
        drawable_to_paths(&linear)[0].d,
        fixture_case_path(&fixture, "linear_path_seed_42")?
    );
    Ok(())
}

#[test]
fn arc_and_curve_render_to_svg_paths() {
    let generator = Generator::new(Config::default());

    let arc = generator.arc(
        100.0,
        100.0,
        160.0,
        90.0,
        std::f64::consts::PI / 6.0,
        std::f64::consts::PI * 1.35,
        false,
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );
    let curve = generator.curve(
        &[[10.0, 80.0], [40.0, 10.0], [100.0, 110.0], [160.0, 40.0]],
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );

    assert_eq!(arc.shape, ShapeType::Arc);
    assert_eq!(curve.shape, ShapeType::Curve);
    assert!(drawable_to_paths(&arc)[0].d.contains('C'));
    assert!(drawable_to_paths(&curve)[0].d.contains('C'));
}

#[test]
fn remaining_primitives_match_roughjs_fixture_ops() -> TestResult {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json"))?;
    let generator = Generator::new(Config::default());

    let polygon = generator.polygon(
        &[[10.0, 10.0], [140.0, 20.0], [120.0, 90.0], [30.0, 110.0]],
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );
    assert_case_ops(&polygon.sets[0].ops, &fixture, "polygon_seed_42", 0)?;

    let open_arc = generator.arc(
        100.0,
        100.0,
        160.0,
        90.0,
        std::f64::consts::PI / 6.0,
        std::f64::consts::PI * 1.35,
        false,
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );
    assert_case_ops(&open_arc.sets[0].ops, &fixture, "arc_open_seed_42", 0)?;

    let closed_arc = generator.arc(
        100.0,
        100.0,
        160.0,
        90.0,
        std::f64::consts::PI / 6.0,
        std::f64::consts::PI * 1.35,
        true,
        Some(Options {
            seed: Some(42),
            fill: Some("red".to_string()),
            ..Options::default()
        }),
    );
    assert_eq!(closed_arc.sets[0].set_type, OpSetType::FillSketch);
    assert_eq!(closed_arc.sets[1].set_type, OpSetType::Path);
    assert_case_ops(&closed_arc.sets[0].ops, &fixture, "arc_closed_seed_42", 0)?;
    assert_case_ops(&closed_arc.sets[1].ops, &fixture, "arc_closed_seed_42", 1)?;

    let curve = generator.curve(
        &[[10.0, 80.0], [40.0, 10.0], [100.0, 110.0], [160.0, 40.0]],
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );
    assert_case_ops(&curve.sets[0].ops, &fixture, "curve_seed_42", 0)?;
    Ok(())
}

#[test]
fn remaining_primitives_cover_closed_unclosed_and_tiny_inputs() {
    let generator = Generator::new(Config::default());

    let open_arc = generator.arc(0.0, 0.0, 20.0, 10.0, 0.0, 1.0, false, None);
    let closed_arc = generator.arc(
        0.0,
        0.0,
        20.0,
        10.0,
        0.0,
        1.0,
        true,
        Some(Options {
            fill: Some("red".to_string()),
            ..Options::default()
        }),
    );
    let empty_curve = generator.curve(&[], None);
    let point_path = generator.linear_path(&[[1.0, 2.0]], None);

    assert_eq!(open_arc.sets.len(), 1);
    assert_eq!(closed_arc.sets.len(), 2);
    assert!(empty_curve.sets[0].ops.is_empty());
    assert!(point_path.sets[0].ops.is_empty());
}

fn fixture_case_path(fixture: &Value, name: &str) -> Result<String, Box<dyn Error>> {
    let path = fixture["cases"]
        .as_array()
        .ok_or_else(|| io::Error::other("fixture cases should be an array"))?
        .iter()
        .find(|case| case["name"] == name)
        .ok_or_else(|| io::Error::other(format!("missing fixture case {name}")))?["paths"][0]["d"]
        .as_str()
        .ok_or_else(|| io::Error::other(format!("missing SVG path for fixture case {name}")))?;
    Ok(path.to_string())
}

fn assert_case_ops(actual: &[Op], fixture: &Value, name: &str, set_index: usize) -> TestResult {
    let expected_ops = fixture["cases"]
        .as_array()
        .ok_or_else(|| io::Error::other("fixture cases should be an array"))?
        .iter()
        .find(|case| case["name"] == name)
        .ok_or_else(|| io::Error::other(format!("missing fixture case {name}")))?["drawable"]
        ["sets"][set_index]["ops"]
        .as_array()
        .ok_or_else(|| io::Error::other(format!("missing fixture ops for case {name}")))?;

    assert_eq!(actual.len(), expected_ops.len(), "{name} op count drift");
    for (actual, expected) in actual.iter().zip(expected_ops) {
        let op_type = expected["op"]
            .as_str()
            .and_then(expected_op_type)
            .ok_or_else(|| io::Error::other(format!("unexpected op type in case {name}")))?;
        assert_eq!(actual.op, op_type);
        let expected_data = expected["data"]
            .as_array()
            .ok_or_else(|| io::Error::other(format!("missing op data in case {name}")))?;
        assert_eq!(actual.data.len(), expected_data.len());
        for (actual_value, expected_value) in actual.data.iter().zip(expected_data) {
            let expected_value = expected_value
                .as_f64()
                .ok_or_else(|| io::Error::other(format!("non-numeric op data in case {name}")))?;
            assert_relative_eq!(*actual_value, expected_value, epsilon = 1e-10);
        }
    }
    Ok(())
}

fn expected_op_type(value: &str) -> Option<OpType> {
    match value {
        "move" => Some(OpType::Move),
        "bcurveTo" => Some(OpType::BCurveTo),
        "lineTo" => Some(OpType::LineTo),
        _ => None,
    }
}
