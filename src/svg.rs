use crate::core::{OpSet, OpType};

pub fn ops_to_path(drawing: &OpSet, fixed_decimals: Option<usize>) -> String {
    drawing
        .ops
        .iter()
        .map(|op| {
            let data = format_data(&op.data, fixed_decimals);
            match op.op {
                OpType::Move => format!("M{} {}", data[0], data[1]),
                OpType::BCurveTo => format!(
                    "C{} {}, {} {}, {} {}",
                    data[0], data[1], data[2], data[3], data[4], data[5]
                ),
                OpType::LineTo => format!("L{} {}", data[0], data[1]),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_data(data: &[f64], fixed_decimals: Option<usize>) -> Vec<String> {
    data.iter()
        .map(|value| match fixed_decimals {
            Some(digits) => format!("{value:.digits$}"),
            None => value.to_string(),
        })
        .collect()
}
