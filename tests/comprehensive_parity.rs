#![cfg(feature = "svg_path")]

use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, Drawable, FillStyle, Generator, Op, OpSetType, OpType, Options, PathInfo};
use serde_json::Value;
use std::error::Error;
use std::io;
use std::str::FromStr;

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
        let drawable = render_fixture_case(&generator, case)
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
            if case["strictOps"].as_bool().unwrap_or(true) {
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

fn render_fixture_case(generator: &Generator, case: &Value) -> Option<Drawable> {
    let method = case["method"].as_str()?;
    let args = case["args"].as_array()?;
    let options = parse_options(&case["options"])?;

    match method {
        "line" => Some(generator.line(
            arg_f64(args, 0)?,
            arg_f64(args, 1)?,
            arg_f64(args, 2)?,
            arg_f64(args, 3)?,
            Some(options),
        )),
        "rectangle" => Some(generator.rectangle(
            arg_f64(args, 0)?,
            arg_f64(args, 1)?,
            arg_f64(args, 2)?,
            arg_f64(args, 3)?,
            Some(options),
        )),
        "ellipse" => Some(generator.ellipse(
            arg_f64(args, 0)?,
            arg_f64(args, 1)?,
            arg_f64(args, 2)?,
            arg_f64(args, 3)?,
            Some(options),
        )),
        "circle" => Some(generator.circle(
            arg_f64(args, 0)?,
            arg_f64(args, 1)?,
            arg_f64(args, 2)?,
            Some(options),
        )),
        "polygon" => Some(generator.polygon(&point_array(args.first()?)?, Some(options))),
        "linearPath" => Some(generator.linear_path(&point_array(args.first()?)?, Some(options))),
        "arc" => Some(generator.arc(
            arg_f64(args, 0)?,
            arg_f64(args, 1)?,
            arg_f64(args, 2)?,
            arg_f64(args, 3)?,
            arg_f64(args, 4)?,
            arg_f64(args, 5)?,
            args.get(6)?.as_bool()?,
            Some(options),
        )),
        "curve" => Some(generator.curve(&point_array(args.first()?)?, Some(options))),
        "path" => Some(generator.path(args.first()?.as_str()?, Some(options))),
        _ => None,
    }
}

fn arg_f64(args: &[Value], index: usize) -> Option<f64> {
    args.get(index)?.as_f64()
}

fn point_array(value: &Value) -> Option<Vec<[f64; 2]>> {
    value
        .as_array()?
        .iter()
        .map(|point| {
            let point = point.as_array()?;
            Some([point.first()?.as_f64()?, point.get(1)?.as_f64()?])
        })
        .collect()
}

fn parse_options(value: &Value) -> Option<Options> {
    let mut options = Options::default();
    let object = value.as_object()?;

    options.max_randomness_offset = f64_option(object.get("maxRandomnessOffset"));
    options.roughness = f64_option(object.get("roughness"));
    options.bowing = f64_option(object.get("bowing"));
    options.stroke = string_option(object.get("stroke"));
    options.stroke_width = f64_option(object.get("strokeWidth"));
    options.curve_fitting = f64_option(object.get("curveFitting"));
    options.curve_tightness = f64_option(object.get("curveTightness"));
    options.curve_step_count = f64_option(object.get("curveStepCount"));
    options.fill = string_option(object.get("fill"));
    options.fill_style = object
        .get("fillStyle")
        .and_then(Value::as_str)
        .and_then(|value| FillStyle::from_str(value).ok());
    options.fill_weight = f64_option(object.get("fillWeight"));
    options.hachure_angle = f64_option(object.get("hachureAngle"));
    options.hachure_gap = f64_option(object.get("hachureGap"));
    options.simplification = f64_option(object.get("simplification"));
    options.dash_offset = f64_option(object.get("dashOffset"));
    options.dash_gap = f64_option(object.get("dashGap"));
    options.zigzag_offset = f64_option(object.get("zigzagOffset"));
    options.seed = object.get("seed").and_then(Value::as_u64);
    options.stroke_line_dash = f64_vec_option(object.get("strokeLineDash"));
    options.stroke_line_dash_offset = f64_option(object.get("strokeLineDashOffset"));
    options.fill_line_dash = f64_vec_option(object.get("fillLineDash"));
    options.fill_line_dash_offset = f64_option(object.get("fillLineDashOffset"));
    options.disable_multi_stroke = bool_option(object.get("disableMultiStroke"));
    options.disable_multi_stroke_fill = bool_option(object.get("disableMultiStrokeFill"));
    options.preserve_vertices = bool_option(object.get("preserveVertices"));
    options.fixed_decimal_place_digits = object
        .get("fixedDecimalPlaceDigits")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok());
    options.fill_shape_roughness_gain = f64_option(object.get("fillShapeRoughnessGain"));

    Some(options)
}

fn f64_option(value: Option<&Value>) -> Option<f64> {
    value.and_then(Value::as_f64)
}

fn string_option(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToString::to_string)
}

fn bool_option(value: Option<&Value>) -> Option<bool> {
    value.and_then(Value::as_bool)
}

fn f64_vec_option(value: Option<&Value>) -> Option<Vec<f64>> {
    value?
        .as_array()?
        .iter()
        .map(Value::as_f64)
        .collect::<Option<Vec<_>>>()
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
