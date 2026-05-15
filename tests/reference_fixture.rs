use serde_json::Value;

#[test]
fn reference_fixture_contains_required_parity_matrix() {
    let fixture = include_str!("fixtures/reference.json");
    let parsed: Value = serde_json::from_str(fixture).expect("reference fixture should be JSON");

    assert_eq!(parsed["roughVersion"], "4.6.6");
    assert_eq!(parsed["source"], "legacy/rough");

    for seed in ["1", "42", "12345", "2147483647"] {
        let sequence = parsed["rng"][seed]
            .as_array()
            .unwrap_or_else(|| panic!("missing rng sequence for seed {seed}"));
        assert_eq!(sequence.len(), 20);
    }

    let cases = parsed["cases"]
        .as_array()
        .expect("reference fixture should contain cases");
    let names = cases
        .iter()
        .map(|case| case["name"].as_str().expect("case should have a name"))
        .collect::<Vec<_>>();

    for required in [
        "line_seed_1",
        "rectangle_seed_42",
        "ellipse_seed_99",
        "circle_seed_12345",
        "polygon_seed_42",
        "linear_path_seed_42",
        "arc_open_seed_42",
        "arc_closed_seed_42",
        "curve_seed_42",
        "svg_path_arc_seed_42",
        "rectangle_fill_hachure",
        "rectangle_fill_solid",
        "rectangle_fill_zigzag",
        "rectangle_fill_cross_hatch",
        "rectangle_fill_dots",
        "rectangle_fill_dashed",
        "rectangle_fill_zigzag_line",
    ] {
        assert!(names.contains(&required), "missing fixture case {required}");
    }
}
