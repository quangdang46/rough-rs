#![cfg(feature = "svg_path")]

use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, Drawable, FillStyle, Generator, Op, OpSetType, OpType, Options, PathInfo};
use serde_json::Value;
use std::error::Error;
use std::io;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn all_reference_fixture_cases_render_with_expected_structure() -> TestResult {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/reference.json"))?;
    let generator = Generator::new(Config::default());

    for case in fixture["cases"]
        .as_array()
        .ok_or_else(|| io::Error::other("fixture cases should be an array"))?
    {
        let name = case["name"]
            .as_str()
            .ok_or_else(|| io::Error::other("fixture case should have a name"))?;
        let drawable = render_fixture_case(&generator, name)
            .ok_or_else(|| io::Error::other(format!("missing renderer for fixture {name}")))?;
        let expected_sets = case["drawable"]["sets"]
            .as_array()
            .ok_or_else(|| io::Error::other(format!("fixture {name} should have sets")))?;

        assert_eq!(
            drawable.shape.as_rough_str(),
            case["drawable"]["shape"].as_str().unwrap_or_default(),
            "{name} shape drift"
        );
        assert_eq!(
            drawable.sets.len(),
            expected_sets.len(),
            "{name} set count drift"
        );
        for (actual, expected) in drawable.sets.iter().zip(expected_sets) {
            assert_eq!(
                actual.set_type,
                expected_set_type(expected["type"].as_str().unwrap_or_default())?,
                "{name} set type drift"
            );
            assert_eq!(
                actual.ops.len(),
                expected["ops"].as_array().map_or(0, Vec::len),
                "{name} op count drift"
            );
            if name != "rectangle_fill_dots" {
                assert_ops_match_fixture(name, &actual.ops, expected["ops"].as_array())?;
            }
            assert!(
                actual
                    .ops
                    .iter()
                    .flat_map(|op| op.data.iter())
                    .all(|value| value.is_finite()),
                "{name} produced non-finite op data"
            );
        }

        for path in drawable_to_paths(&drawable) {
            assert_valid_path_info(name, &path);
        }
    }

    Ok(())
}

#[test]
fn seeded_rendering_is_deterministic_across_option_matrix() {
    let generator = Generator::new(Config::default());
    let option_matrix = [
        Options {
            seed: Some(1),
            roughness: Some(0.0),
            ..Options::default()
        },
        Options {
            seed: Some(42),
            roughness: Some(1.0),
            stroke_width: Some(2.0),
            ..Options::default()
        },
        Options {
            seed: Some(12345),
            fill: Some("red".to_string()),
            fill_style: Some(FillStyle::CrossHatch),
            hachure_gap: Some(6.0),
            ..Options::default()
        },
    ];

    for options in option_matrix {
        let first = generator.rectangle(10.0, 20.0, 120.0, 80.0, Some(options.clone()));
        let second = generator.rectangle(10.0, 20.0, 120.0, 80.0, Some(options));

        assert_eq!(first.sets, second.sets);
        assert_eq!(drawable_to_paths(&first), drawable_to_paths(&second));
    }
}

#[test]
fn e2e_svg_sample_contains_valid_embeddable_paths() {
    let generator = Generator::new(Config::default());
    let mut path_elements = Vec::new();
    for drawable in [
        generator.line(
            10.0,
            10.0,
            160.0,
            30.0,
            Some(Options {
                seed: Some(1),
                ..Options::default()
            }),
        ),
        generator.rectangle(
            20.0,
            50.0,
            90.0,
            60.0,
            Some(Options {
                seed: Some(2),
                fill: Some("#f5d76e".to_string()),
                fill_style: Some(FillStyle::ZigzagLine),
                ..Options::default()
            }),
        ),
        generator.ellipse(
            170.0,
            90.0,
            80.0,
            50.0,
            Some(Options {
                seed: Some(3),
                ..Options::default()
            }),
        ),
        generator.path(
            "M40 160 C 80 120, 140 200, 180 150 A 30 20 0 0 1 230 170",
            Some(Options {
                seed: Some(4),
                stroke: Some("#365".to_string()),
                ..Options::default()
            }),
        ),
    ] {
        for path in drawable_to_paths(&drawable) {
            assert_valid_path_info("e2e_svg_sample", &path);
            path_elements.push(format!(
                r#"<path d="{}" stroke="{}" stroke-width="{}" fill="{}"/>"#,
                path.d, path.stroke, path.stroke_width, path.fill
            ));
        }
    }

    let svg = format!(
        r#"<svg xmlns="{}" viewBox="0 0 260 220">{}</svg>"#,
        rough_rs::core::SVG_NS,
        path_elements.join("")
    );

    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("<path"));
    assert!(svg.ends_with("</svg>"));
}

fn render_fixture_case(generator: &Generator, name: &str) -> Option<Drawable> {
    let drawable = match name {
        "line_seed_1" => generator.line(
            10.0,
            10.0,
            200.0,
            100.0,
            Some(Options {
                seed: Some(1),
                ..Options::default()
            }),
        ),
        "rectangle_seed_42" => generator.rectangle(
            10.0,
            10.0,
            200.0,
            100.0,
            Some(Options {
                seed: Some(42),
                ..Options::default()
            }),
        ),
        "ellipse_seed_99" => generator.ellipse(
            100.0,
            100.0,
            200.0,
            150.0,
            Some(Options {
                seed: Some(99),
                ..Options::default()
            }),
        ),
        "circle_seed_12345" => generator.circle(
            100.0,
            100.0,
            80.0,
            Some(Options {
                seed: Some(12345),
                ..Options::default()
            }),
        ),
        "polygon_seed_42" => generator.polygon(
            &[[10.0, 10.0], [140.0, 20.0], [120.0, 90.0], [30.0, 110.0]],
            Some(Options {
                seed: Some(42),
                ..Options::default()
            }),
        ),
        "linear_path_seed_42" => generator.linear_path(
            &[[10.0, 10.0], [40.0, 70.0], [100.0, 30.0], [160.0, 90.0]],
            Some(Options {
                seed: Some(42),
                ..Options::default()
            }),
        ),
        "arc_open_seed_42" => generator.arc(
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
        ),
        "arc_closed_seed_42" => generator.arc(
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
        ),
        "curve_seed_42" => generator.curve(
            &[[10.0, 80.0], [40.0, 10.0], [100.0, 110.0], [160.0, 40.0]],
            Some(Options {
                seed: Some(42),
                ..Options::default()
            }),
        ),
        "svg_path_arc_seed_42" => generator.path(
            "M80 80 A 45 45, 0, 0, 0, 125 125 L 125 80 Z",
            Some(Options {
                seed: Some(42),
                fill: Some("green".to_string()),
                ..Options::default()
            }),
        ),
        name if name.starts_with("rectangle_fill_") => {
            let fill_style = match name {
                "rectangle_fill_hachure" => FillStyle::Hachure,
                "rectangle_fill_solid" => FillStyle::Solid,
                "rectangle_fill_zigzag" => FillStyle::Zigzag,
                "rectangle_fill_cross_hatch" => FillStyle::CrossHatch,
                "rectangle_fill_dots" => FillStyle::Dots,
                "rectangle_fill_dashed" => FillStyle::Dashed,
                "rectangle_fill_zigzag_line" => FillStyle::ZigzagLine,
                _ => return None,
            };
            generator.rectangle(
                10.0,
                10.0,
                120.0,
                80.0,
                Some(Options {
                    seed: Some(777),
                    fill: Some("red".to_string()),
                    fill_style: Some(fill_style),
                    ..Options::default()
                }),
            )
        }
        _ => return None,
    };
    Some(drawable)
}

fn expected_set_type(value: &str) -> Result<OpSetType, io::Error> {
    match value {
        "path" => Ok(OpSetType::Path),
        "fillPath" => Ok(OpSetType::FillPath),
        "fillSketch" => Ok(OpSetType::FillSketch),
        _ => Err(io::Error::other(format!("unexpected set type {value}"))),
    }
}

fn assert_ops_match_fixture(
    name: &str,
    actual: &[Op],
    expected: Option<&Vec<Value>>,
) -> TestResult {
    let expected = expected.ok_or_else(|| io::Error::other(format!("missing ops for {name}")))?;
    for (actual, expected) in actual.iter().zip(expected) {
        let expected_op = expected["op"]
            .as_str()
            .and_then(expected_op_type)
            .ok_or_else(|| io::Error::other(format!("unexpected op type in {name}")))?;
        assert_eq!(actual.op, expected_op, "{name} op type drift");
        let expected_data = expected["data"]
            .as_array()
            .ok_or_else(|| io::Error::other(format!("missing op data in {name}")))?;
        assert_eq!(
            actual.data.len(),
            expected_data.len(),
            "{name} data length drift"
        );
        for (actual_value, expected_value) in actual.data.iter().zip(expected_data) {
            let expected_value = expected_value
                .as_f64()
                .ok_or_else(|| io::Error::other(format!("non-numeric op data in {name}")))?;
            assert!(
                (*actual_value - expected_value).abs() <= 1e-10,
                "{name} numeric drift: actual {actual_value}, expected {expected_value}"
            );
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

fn assert_valid_path_info(name: &str, path: &PathInfo) {
    assert!(!path.d.contains("NaN"), "{name} path contains NaN");
    assert!(!path.d.contains("inf"), "{name} path contains inf");
    assert!(
        path.d.is_empty() || path.d.starts_with('M') || path.d.starts_with('L'),
        "{name} path has unexpected command start: {}",
        path.d
    );
    assert!(path.stroke_width >= 0.0, "{name} has negative stroke width");
}
