use rough_rs::svg::drawable_to_paths;
use rough_rs::{Config, Generator, Options, ShapeType};

#[test]
fn generator_ellipse_serializes_svg_path() {
    let generator = Generator::new(Config::default());

    let drawable = generator.ellipse(
        100.0,
        100.0,
        200.0,
        150.0,
        Some(Options {
            seed: Some(99),
            ..Options::default()
        }),
    );
    let paths = drawable_to_paths(&drawable);

    assert_eq!(drawable.shape, ShapeType::Ellipse);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].stroke, "#000");
    assert_eq!(paths[0].fill, "none");
    assert!(paths[0].d.starts_with('M'));
    assert!(paths[0].d.contains('C'));
}

#[test]
fn generator_circle_delegates_to_ellipse_geometry() {
    let generator = Generator::new(Config::default());

    let circle = generator.circle(
        100.0,
        100.0,
        80.0,
        Some(Options {
            seed: Some(12345),
            ..Options::default()
        }),
    );
    let ellipse = generator.ellipse(
        100.0,
        100.0,
        80.0,
        80.0,
        Some(Options {
            seed: Some(12345),
            ..Options::default()
        }),
    );

    assert_eq!(circle.shape, ShapeType::Circle);
    assert_eq!(circle.sets, ellipse.sets);
}
