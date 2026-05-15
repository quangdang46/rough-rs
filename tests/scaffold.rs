use rough_rs::{Config, Generator, ShapeType};

#[test]
fn public_modules_are_reachable() {
    let generator = Generator::new(Config::default());
    let drawable = generator.empty(ShapeType::Line);

    assert_eq!(drawable.sets.len(), 1);
    assert_eq!(drawable.shape, ShapeType::Line);
}
