use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rough_rs::{Config, FillStyle, Generator, Options};

fn bench_primitives(c: &mut Criterion) {
    let generator = Generator::new(Config::default());

    c.bench_function("rectangle", |b| {
        b.iter(|| {
            generator.rectangle(
                black_box(10.0),
                black_box(10.0),
                black_box(200.0),
                black_box(100.0),
                Some(Options {
                    seed: Some(42),
                    ..Options::default()
                }),
            )
        });
    });

    c.bench_function("ellipse", |b| {
        b.iter(|| {
            generator.ellipse(
                black_box(100.0),
                black_box(100.0),
                black_box(200.0),
                black_box(150.0),
                Some(Options {
                    seed: Some(99),
                    ..Options::default()
                }),
            )
        });
    });

    c.bench_function("complex_svg_path", |b| {
        b.iter(|| {
            generator.path(
                black_box("M20 80 C 40 10, 65 10, 95 80 S 150 150, 180 80 A 30 25 20 0 1 230 120 L 210 160 Z"),
                Some(Options {
                    seed: Some(777),
                    fill: Some("red".to_string()),
                    ..Options::default()
                }),
            )
        });
    });

    c.bench_function("large_hachure_fill", |b| {
        let points = [
            [0.0, 0.0],
            [240.0, 20.0],
            [280.0, 180.0],
            [180.0, 280.0],
            [20.0, 220.0],
            [-40.0, 90.0],
        ];
        b.iter(|| {
            generator.polygon(
                black_box(&points),
                Some(Options {
                    seed: Some(12345),
                    fill: Some("red".to_string()),
                    fill_style: Some(FillStyle::Hachure),
                    ..Options::default()
                }),
            )
        });
    });
}

criterion_group!(benches, bench_primitives);
criterion_main!(benches);
