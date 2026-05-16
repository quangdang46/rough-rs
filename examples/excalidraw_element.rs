use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, FillStyle, Generator, Options};

struct ExcalidrawLikeElement {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    seed: u64,
    stroke_color: &'static str,
    background_color: &'static str,
    roughness: f64,
}

fn main() {
    let element = ExcalidrawLikeElement {
        x: 40.0,
        y: 30.0,
        width: 180.0,
        height: 110.0,
        seed: 12345,
        stroke_color: "#111827",
        background_color: "#bfdbfe",
        roughness: 1.2,
    };
    let generator = Generator::new(Config::default());
    let drawable = generator.rectangle(
        element.x,
        element.y,
        element.width,
        element.height,
        Some(Options {
            seed: Some(element.seed),
            stroke: Some(element.stroke_color.to_string()),
            fill: Some(element.background_color.to_string()),
            fill_style: Some(FillStyle::CrossHatch),
            roughness: Some(element.roughness),
            stroke_width: Some(2.0),
            ..Options::default()
        }),
    );

    println!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 260 180">"#);
    for path in drawable_to_paths(&drawable) {
        println!(
            r#"<path d="{}" stroke="{}" stroke-width="{}" fill="{}"/>"#,
            path.d, path.stroke, path.stroke_width, path.fill
        );
    }
    println!("</svg>");
}
