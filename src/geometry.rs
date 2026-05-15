pub type Point = [f64; 2];
pub type Line = [Point; 2];

pub fn line_length(line: Line) -> f64 {
    let dx = line[0][0] - line[1][0];
    let dy = line[0][1] - line[1][1];
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_length_uses_euclidean_distance() {
        assert_eq!(line_length([[0.0, 0.0], [3.0, 4.0]]), 5.0);
    }
}
