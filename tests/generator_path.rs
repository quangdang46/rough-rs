#![cfg(feature = "svg_path")]

use approx::assert_relative_eq;
use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, Generator, Op, OpSetType, OpType, Options, ShapeType};
use serde_json::Value;
use std::error::Error;
use std::io;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn generator_svg_path_arc_fixture_matches_legacy_ops() -> TestResult {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json"))?;
    let generator = Generator::new(Config::default());

    let drawable = generator.path(
        "M80 80 A 45 45, 0, 0, 0, 125 125 L 125 80 Z",
        Some(Options {
            seed: Some(42),
            fill: Some("green".to_string()),
            ..Options::default()
        }),
    );

    assert_eq!(drawable.shape, ShapeType::Path);
    assert_eq!(drawable.sets[0].set_type, OpSetType::FillSketch);
    assert_eq!(drawable.sets[1].set_type, OpSetType::Path);
    assert_case_ops(&drawable.sets[0].ops, &fixture, "svg_path_arc_seed_42", 0)?;
    assert_case_ops(&drawable.sets[1].ops, &fixture, "svg_path_arc_seed_42", 1)?;
    Ok(())
}

#[test]
fn generator_svg_path_supports_normalized_commands_and_relative_input() {
    let generator = Generator::new(Config::default());
    let drawable = generator.path(
        "m10 10 h30 v20 l20 -10 c5 10 15 10 20 0 s20 -10 25 0 q10 20 20 0 t20 0 a15 10 30 0 1 30 20 z",
        Some(Options {
            seed: Some(7),
            ..Options::default()
        }),
    );
    let paths = drawable_to_paths(&drawable);

    assert_eq!(drawable.shape, ShapeType::Path);
    assert_eq!(drawable.sets.len(), 1);
    assert!(paths[0].d.starts_with('M'));
    assert!(paths[0].d.contains('C'));
}

#[test]
fn generator_svg_path_handles_empty_malformed_and_simplified_paths() {
    let generator = Generator::new(Config::default());

    let empty = generator.path("", None);
    let malformed = generator.path("L 10 10", None);
    let simplified = generator.path(
        "M0 0 C 25 50, 75 -50, 100 0",
        Some(Options {
            seed: Some(9),
            simplification: Some(0.5),
            ..Options::default()
        }),
    );

    assert!(empty.sets.is_empty());
    assert!(malformed.sets.is_empty());
    assert_eq!(simplified.shape, ShapeType::Path);
    assert!(!simplified.sets.is_empty());
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
