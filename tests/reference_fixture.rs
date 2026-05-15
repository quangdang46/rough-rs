use serde_json::Value;

#[test]
fn reference_fixture_contains_required_parity_matrix() {
    let fixture = include_str!("fixtures/reference.json");
    let parsed: Value = serde_json::from_str(fixture).expect("reference fixture should be JSON");

    assert_eq!(parsed["roughVersion"], "4.6.6");
    assert_eq!(parsed["source"], "legacy/rough");
    assert!(parsed["curveUtilities"]["curveToBezier"].is_array());
    assert!(parsed["curveUtilities"]["pointsOnBezierCurves"].is_array());
    assert!(parsed["curveUtilities"]["simplifiedPointsOnBezierCurves"].is_array());

    for seed in ["1", "42", "12345", "2147483647"] {
        let sequence = parsed["rng"][seed]
            .as_array()
            .unwrap_or_else(|| panic!("missing rng sequence for seed {seed}"));
        assert_eq!(sequence.len(), 20);
    }

    let cases = parsed["cases"]
        .as_array()
        .expect("reference fixture should contain cases");
    assert!(
        cases.len() >= 37,
        "reference fixture should cover the expanded parity matrix"
    );
    let names = cases
        .iter()
        .map(|case| case["name"].as_str().expect("case should have a name"))
        .collect::<Vec<_>>();

    for required in [
        "line_seed_1",
        "rectangle_seed_42",
        "ellipse_seed_99",
        "ellipse_solid_fill_seed_42",
        "ellipse_hachure_fill_seed_42",
        "circle_seed_12345",
        "circle_dots_fill_seed_42",
        "polygon_seed_42",
        "linear_path_seed_42",
        "arc_open_seed_42",
        "arc_closed_seed_42",
        "curve_seed_42",
        "svg_path_arc_seed_42",
        "line_roughness_zero_seed_42",
        "line_disable_multistroke_seed_42",
        "line_preserve_vertices_seed_42",
        "rectangle_stroke_none_solid_fill_seed_42",
        "rectangle_fill_none_seed_42",
        "rectangle_hachure_angle_zero_gap_seed_42",
        "rectangle_custom_dash_fill_seed_42",
        "rectangle_custom_zigzag_fill_seed_42",
        "ellipse_negative_dimensions_seed_42",
        "ellipse_tiny_seed_42",
        "polygon_concave_hachure_seed_42",
        "arc_closed_hachure_seed_42",
        "curve_three_points_seed_42",
        "curve_repeated_points_seed_42",
        "svg_path_relative_commands_seed_42",
        "svg_path_simplification_seed_42",
        "rectangle_seed_zero_structure",
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
