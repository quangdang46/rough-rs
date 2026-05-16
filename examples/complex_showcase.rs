use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, FillStyle, Generator, Options};
use std::fmt::Write;

fn main() {
    let generator = Generator::new(Config::default());
    let shapes = [
        generator.polygon(
            &[
                [35.0, 35.0],
                [145.0, 35.0],
                [110.0, 85.0],
                [145.0, 135.0],
                [35.0, 135.0],
                [70.0, 85.0],
            ],
            Some(Options {
                seed: Some(11),
                stroke: Some("#7c2d12".to_string()),
                fill: Some("#fed7aa".to_string()),
                fill_style: Some(FillStyle::CrossHatch),
                stroke_width: Some(2.0),
                ..Options::default()
            }),
        ),
        generator.arc(
            260.0,
            85.0,
            150.0,
            105.0,
            0.15,
            std::f64::consts::PI * 1.65,
            true,
            Some(Options {
                seed: Some(22),
                stroke: Some("#075985".to_string()),
                fill: Some("#bae6fd".to_string()),
                fill_style: Some(FillStyle::ZigzagLine),
                stroke_width: Some(2.0),
                ..Options::default()
            }),
        ),
        generator.curve(
            &[
                [35.0, 245.0],
                [95.0, 165.0],
                [155.0, 250.0],
                [225.0, 175.0],
                [305.0, 250.0],
            ],
            Some(Options {
                seed: Some(33),
                stroke: Some("#4c1d95".to_string()),
                stroke_width: Some(3.0),
                roughness: Some(1.4),
                ..Options::default()
            }),
        ),
        generator.path(
            "M420 55 C470 15 540 35 552 92 C565 154 500 177 455 142 C410 178 350 145 368 86 C374 65 392 55 420 55 Z",
            Some(Options {
                seed: Some(44),
                stroke: Some("#166534".to_string()),
                fill: Some("#bbf7d0".to_string()),
                fill_style: Some(FillStyle::Dots),
                stroke_width: Some(2.0),
                ..Options::default()
            }),
        ),
        generator.path(
            "M430 230 L462 170 L494 230 L560 238 L512 282 L526 348 L462 315 L398 348 L412 282 L364 238 Z",
            Some(Options {
                seed: Some(55),
                stroke: Some("#be123c".to_string()),
                fill: Some("#fecdd3".to_string()),
                fill_style: Some(FillStyle::Dashed),
                stroke_width: Some(2.0),
                hachure_gap: Some(6.0),
                ..Options::default()
            }),
        ),
    ];

    let mut body = String::new();
    for shape in shapes {
        for path in drawable_to_paths(&shape) {
            let _ = write!(
                &mut body,
                r#"<path d="{}" stroke="{}" stroke-width="{}" fill="{}"/>"#,
                path.d, path.stroke, path.stroke_width, path.fill
            );
        }
    }

    println!(
        r#"<svg xmlns="{}" viewBox="0 0 600 370">{}</svg>"#,
        rough_rs::core::SVG_NS,
        body
    );
}
