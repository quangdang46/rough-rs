use approx::assert_relative_eq;
use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, FillStyle, Generator, OpType, Options, ShapeType};
use serde_json::Value;

#[test]
fn generator_line_returns_line_drawable() {
    let generator = Generator::new(Config::default());
    let drawable = generator.line(
        10.0,
        10.0,
        20.0,
        20.0,
        Some(Options {
            seed: Some(1),
            ..Options::default()
        }),
    );

    assert_eq!(drawable.shape, ShapeType::Line);
    assert_eq!(drawable.sets.len(), 1);
    assert_eq!(drawable.sets[0].ops[1].op, OpType::BCurveTo);
}

#[test]
fn generator_rectangle_strokes_outline_and_serializes_to_svg_path() {
    let generator = Generator::new(Config::default());
    let drawable = generator.rectangle(
        10.0,
        10.0,
        200.0,
        100.0,
        Some(Options {
            seed: Some(42),
            stroke: Some("#123".to_string()),
            stroke_width: Some(2.0),
            ..Options::default()
        }),
    );

    let paths = drawable_to_paths(&drawable);

    assert_eq!(drawable.shape, ShapeType::Rectangle);
    assert_eq!(drawable.sets.len(), 1);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].stroke, "#123");
    assert_eq!(paths[0].stroke_width, 2.0);
    assert_eq!(paths[0].fill, "none");
    assert!(paths[0].d.starts_with('M'));
}

#[test]
fn generator_rectangle_respects_none_stroke() {
    let generator = Generator::new(Config::default());
    let drawable = generator.rectangle(
        10.0,
        10.0,
        200.0,
        100.0,
        Some(Options {
            stroke: Some("none".to_string()),
            ..Options::default()
        }),
    );

    assert!(drawable.sets.is_empty());
}

#[test]
fn generator_rectangle_matches_legacy_fixture_path() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json")).unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "rectangle_seed_42")
        .expect("rectangle fixture should exist");
    let expected_path = case["paths"][0]["d"].as_str().unwrap();
    let generator = Generator::new(Config::default());

    let drawable = generator.rectangle(
        10.0,
        10.0,
        200.0,
        100.0,
        Some(Options {
            seed: Some(42),
            ..Options::default()
        }),
    );
    let paths = drawable_to_paths(&drawable);

    assert_eq!(paths[0].d, expected_path);
}

#[test]
fn generator_rectangle_hachure_fill_matches_legacy_fixture_ops() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json")).unwrap();
    let case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "rectangle_fill_hachure")
        .expect("hachure fixture should exist");
    let expected_ops = case["drawable"]["sets"][0]["ops"].as_array().unwrap();
    let generator = Generator::new(Config::default());

    let drawable = generator.rectangle(
        10.0,
        10.0,
        120.0,
        80.0,
        Some(Options {
            seed: Some(777),
            fill: Some("red".to_string()),
            fill_style: Some(FillStyle::Hachure),
            ..Options::default()
        }),
    );

    assert_eq!(drawable.sets[0].set_type, rough_rs::OpSetType::FillSketch);
    assert_eq!(drawable.sets[0].ops.len(), expected_ops.len());
    for (actual, expected) in drawable.sets[0].ops.iter().zip(expected_ops) {
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
fn generator_rectangle_solid_fill_maps_to_fill_path() {
    let generator = Generator::new(Config::default());
    let drawable = generator.rectangle(
        10.0,
        10.0,
        120.0,
        80.0,
        Some(Options {
            seed: Some(777),
            fill: Some("red".to_string()),
            fill_style: Some(FillStyle::Solid),
            ..Options::default()
        }),
    );
    let paths = drawable_to_paths(&drawable);

    assert_eq!(drawable.sets[0].set_type, rough_rs::OpSetType::FillPath);
    assert_eq!(paths[0].stroke, "none");
    assert_eq!(paths[0].fill, "red");
}
