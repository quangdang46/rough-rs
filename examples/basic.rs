use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, FillStyle, Generator, Options};
use std::fmt::Write;

fn main() {
    let generator = Generator::new(Config::default());
    let shapes = [
        generator.rectangle(
            20.0,
            45.0,
            120.0,
            80.0,
            Some(Options {
                seed: Some(2),
                fill: Some("#facc15".to_string()),
                fill_style: Some(FillStyle::Hachure),
                ..Options::default()
            }),
        ),
        generator.circle(
            210.0,
            85.0,
            80.0,
            Some(Options {
                seed: Some(3),
                stroke: Some("#2563eb".to_string()),
                fill: Some("#bfdbfe".to_string()),
                fill_style: Some(FillStyle::Hachure),
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
        r#"<svg xmlns="{}" viewBox="0 0 280 170">{}</svg>"#,
        rough_rs::core::SVG_NS,
        body
    );
}
