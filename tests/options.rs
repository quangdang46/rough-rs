use std::str::FromStr;

use rough_rs::{Config, FillStyle, Generator, Options, ResolvedOptions, ShapeType};

#[test]
fn roughjs_default_options_match_legacy_generator() {
    let options = ResolvedOptions::default();

    assert_eq!(options.max_randomness_offset, 2.0);
    assert_eq!(options.roughness, 1.0);
    assert_eq!(options.bowing, 1.0);
    assert_eq!(options.stroke, "#000");
    assert_eq!(options.stroke_width, 1.0);
    assert_eq!(options.curve_tightness, 0.0);
    assert_eq!(options.curve_fitting, 0.95);
    assert_eq!(options.curve_step_count, 9.0);
    assert_eq!(options.fill_style, FillStyle::Hachure);
    assert_eq!(options.fill_weight, -1.0);
    assert_eq!(options.hachure_angle, -41.0);
    assert_eq!(options.hachure_gap, -1.0);
    assert_eq!(options.dash_offset, -1.0);
    assert_eq!(options.dash_gap, -1.0);
    assert_eq!(options.zigzag_offset, -1.0);
    assert_eq!(options.seed, 0);
    assert!(!options.disable_multi_stroke);
    assert!(!options.disable_multi_stroke_fill);
    assert!(!options.preserve_vertices);
    assert_eq!(options.fixed_decimal_place_digits, None);
    assert_eq!(options.stroke_line_dash, None);
    assert_eq!(options.stroke_line_dash_offset, None);
    assert_eq!(options.fill_line_dash, None);
    assert_eq!(options.fill_line_dash_offset, None);
    assert_eq!(options.fill_shape_roughness_gain, 0.8);
}

#[test]
fn effective_defaults_follow_roughjs_sentinel_rules() {
    let options = ResolvedOptions {
        stroke_width: 3.0,
        ..ResolvedOptions::default()
    };

    assert_eq!(options.effective_fill_weight(), 1.5);
    assert_eq!(options.effective_hachure_gap(), 12.0);
    assert_eq!(options.effective_dash_offset(), 12.0);
    assert_eq!(options.effective_dash_gap(), 12.0);
    assert_eq!(options.effective_zigzag_offset(), 12.0);
}

#[test]
fn config_and_per_call_options_merge_over_defaults() {
    let generator = Generator::new(Config {
        options: Some(Options {
            stroke: Some("#123".to_string()),
            roughness: Some(2.0),
            seed: Some(42),
            ..Options::default()
        }),
    });

    assert_eq!(generator.default_options().stroke, "#123");
    assert_eq!(generator.default_options().roughness, 2.0);
    assert_eq!(generator.default_options().seed, 42);

    let resolved = generator.resolve_options(Some(&Options {
        stroke_width: Some(5.0),
        fill_style: Some(FillStyle::Dots),
        fixed_decimal_place_digits: Some(2),
        stroke_line_dash: Some(vec![4.0, 2.0]),
        stroke_line_dash_offset: Some(1.0),
        fill_line_dash: Some(vec![3.0, 1.0]),
        fill_line_dash_offset: Some(2.0),
        ..Options::default()
    }));

    assert_eq!(resolved.stroke, "#123");
    assert_eq!(resolved.roughness, 2.0);
    assert_eq!(resolved.stroke_width, 5.0);
    assert_eq!(resolved.fill_style, FillStyle::Dots);
    assert_eq!(resolved.fixed_decimal_place_digits, Some(2));
    assert_eq!(resolved.stroke_line_dash, Some(vec![4.0, 2.0]));
    assert_eq!(resolved.stroke_line_dash_offset, Some(1.0));
    assert_eq!(resolved.fill_line_dash, Some(vec![3.0, 1.0]));
    assert_eq!(resolved.fill_line_dash_offset, Some(2.0));
}

#[test]
fn fill_style_round_trips_roughjs_names() {
    let cases = [
        ("hachure", FillStyle::Hachure),
        ("solid", FillStyle::Solid),
        ("zigzag", FillStyle::Zigzag),
        ("cross-hatch", FillStyle::CrossHatch),
        ("dots", FillStyle::Dots),
        ("dashed", FillStyle::Dashed),
        ("zigzag-line", FillStyle::ZigzagLine),
    ];

    for (name, style) in cases {
        assert_eq!(FillStyle::from_str(name).unwrap(), style);
        assert_eq!(style.as_rough_str(), name);
        assert_eq!(style.to_string(), name);
    }

    assert!(FillStyle::from_str("sunburst").is_err());
}

#[test]
fn shape_type_uses_roughjs_public_names() {
    assert_eq!(ShapeType::Line.to_string(), "line");
    assert_eq!(ShapeType::Rectangle.to_string(), "rectangle");
    assert_eq!(ShapeType::Ellipse.to_string(), "ellipse");
    assert_eq!(ShapeType::Circle.to_string(), "circle");
    assert_eq!(ShapeType::LinearPath.to_string(), "linearPath");
    assert_eq!(ShapeType::Arc.to_string(), "arc");
    assert_eq!(ShapeType::Curve.to_string(), "curve");
    assert_eq!(ShapeType::Polygon.to_string(), "polygon");
    assert_eq!(ShapeType::Path.to_string(), "path");
}
