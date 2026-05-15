use crate::core::{Drawable, OpSet, OpSetType, OpType, SvgPath};

const NONE: &str = "none";

pub fn ops_to_path(drawing: &OpSet, fixed_decimals: Option<usize>) -> String {
    drawing
        .ops
        .iter()
        .filter_map(|op| {
            let data = format_data(&op.data, fixed_decimals);
            match op.op {
                OpType::Move if data.len() >= 2 => Some(format!("M{} {}", data[0], data[1])),
                OpType::BCurveTo if data.len() >= 6 => Some(format!(
                    "C{} {}, {} {}, {} {}",
                    data[0], data[1], data[2], data[3], data[4], data[5]
                )),
                OpType::LineTo if data.len() >= 2 => Some(format!("L{} {}", data[0], data[1])),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn drawable_to_paths(drawable: &Drawable) -> Vec<SvgPath> {
    drawable
        .sets
        .iter()
        .map(|drawing| match drawing.set_type {
            OpSetType::Path => SvgPath {
                d: ops_to_path(drawing, drawable.options.fixed_decimal_place_digits),
                stroke: drawable.options.stroke.clone(),
                stroke_width: drawable.options.stroke_width,
                fill: NONE.to_string(),
            },
            OpSetType::FillPath => SvgPath {
                d: ops_to_path(drawing, drawable.options.fixed_decimal_place_digits),
                stroke: NONE.to_string(),
                stroke_width: 0.0,
                fill: drawable
                    .options
                    .fill
                    .clone()
                    .unwrap_or_else(|| NONE.to_string()),
            },
            OpSetType::FillSketch => SvgPath {
                d: ops_to_path(drawing, drawable.options.fixed_decimal_place_digits),
                stroke: drawable
                    .options
                    .fill
                    .clone()
                    .unwrap_or_else(|| NONE.to_string()),
                stroke_width: drawable.options.effective_fill_weight(),
                fill: NONE.to_string(),
            },
        })
        .collect()
}

fn format_data(data: &[f64], fixed_decimals: Option<usize>) -> Vec<String> {
    data.iter()
        .map(|value| format_number(*value, fixed_decimals))
        .collect()
}

fn format_number(value: f64, fixed_decimals: Option<usize>) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }

    let normalized = if value == 0.0 { 0.0 } else { value };
    match fixed_decimals {
        Some(digits) => {
            let rounded = format!("{normalized:.digits$}")
                .parse::<f64>()
                .unwrap_or(0.0);
            if rounded == 0.0 {
                "0".to_string()
            } else {
                rounded.to_string()
            }
        }
        None => normalized.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Drawable, Op, OpSet, OpSetType, OpType, ResolvedOptions, ShapeType};

    #[test]
    fn ops_to_path_matches_roughjs_command_format() {
        let drawing = OpSet::new(
            OpSetType::Path,
            vec![
                Op::new(OpType::Move, vec![1.2345, 2.0]),
                Op::new(OpType::BCurveTo, vec![3.0, 4.5, 5.0, 6.25, 7.0, 8.0]),
                Op::new(OpType::LineTo, vec![9.0, 10.0]),
            ],
        );

        assert_eq!(
            ops_to_path(&drawing, None),
            "M1.2345 2 C3 4.5, 5 6.25, 7 8 L9 10"
        );
    }

    #[test]
    fn fixed_decimals_round_and_strip_trailing_zeroes_like_javascript_numbers() {
        let drawing = OpSet::new(
            OpSetType::Path,
            vec![
                Op::new(OpType::Move, vec![1.2345, 2.0001]),
                Op::new(OpType::LineTo, vec![-0.0001, f64::INFINITY]),
            ],
        );

        assert_eq!(ops_to_path(&drawing, Some(2)), "M1.23 2 L0 0");
    }

    #[test]
    fn malformed_ops_are_ignored_instead_of_panicking() {
        let drawing = OpSet::new(
            OpSetType::Path,
            vec![
                Op::new(OpType::Move, vec![1.0]),
                Op::new(OpType::LineTo, vec![2.0, 3.0]),
            ],
        );

        assert_eq!(ops_to_path(&drawing, None), "L2 3");
    }

    #[test]
    fn drawable_to_paths_maps_path_attributes_by_opset_type() {
        let options = ResolvedOptions {
            stroke: "#111".to_string(),
            stroke_width: 3.0,
            fill: Some("red".to_string()),
            fill_weight: -1.0,
            ..ResolvedOptions::default()
        };
        let drawable = Drawable {
            shape: ShapeType::Rectangle,
            options,
            sets: vec![
                OpSet::new(
                    OpSetType::FillPath,
                    vec![Op::new(OpType::Move, vec![0.0, 0.0])],
                ),
                OpSet::new(
                    OpSetType::FillSketch,
                    vec![Op::new(OpType::LineTo, vec![1.0, 1.0])],
                ),
                OpSet::new(
                    OpSetType::Path,
                    vec![Op::new(OpType::LineTo, vec![2.0, 2.0])],
                ),
            ],
        };

        let paths = drawable_to_paths(&drawable);

        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].stroke, NONE);
        assert_eq!(paths[0].stroke_width, 0.0);
        assert_eq!(paths[0].fill, "red");
        assert_eq!(paths[1].stroke, "red");
        assert_eq!(paths[1].stroke_width, 1.5);
        assert_eq!(paths[1].fill, NONE);
        assert_eq!(paths[2].stroke, "#111");
        assert_eq!(paths[2].stroke_width, 3.0);
        assert_eq!(paths[2].fill, NONE);
    }
}
