pub type Point = [f64; 2];
pub type Line = [Point; 2];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn line_length(line: Line) -> f64 {
    let dx = line[0][0] - line[1][0];
    let dy = line[0][1] - line[1][1];
    (dx * dx + dy * dy).sqrt()
}

pub fn rotate_point(point: Point, center: Point, angle: f64) -> Point {
    let (sin_angle, cos_angle) = angle.sin_cos();
    let dx = point[0] - center[0];
    let dy = point[1] - center[1];
    [
        center[0] + dx * cos_angle - dy * sin_angle,
        center[1] + dx * sin_angle + dy * cos_angle,
    ]
}

pub fn rotate_points(points: &[Point], center: Point, angle: f64) -> Vec<Point> {
    points
        .iter()
        .map(|point| rotate_point(*point, center, angle))
        .collect()
}

pub fn rotate_line(line: Line, center: Point, angle: f64) -> Line {
    [
        rotate_point(line[0], center, angle),
        rotate_point(line[1], center, angle),
    ]
}

pub fn rotate_lines(lines: &[Line], center: Point, angle: f64) -> Vec<Line> {
    lines
        .iter()
        .map(|line| rotate_line(*line, center, angle))
        .collect()
}

pub fn line_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let a1 = b[1] - a[1];
    let b1 = a[0] - b[0];
    let c1 = a1 * a[0] + b1 * a[1];

    let a2 = d[1] - c[1];
    let b2 = c[0] - d[0];
    let c2 = a2 * c[0] + b2 * c[1];

    let det = a1 * b2 - a2 * b1;
    if det.abs() < f64::EPSILON {
        return None;
    }

    Some([(b2 * c1 - b1 * c2) / det, (a1 * c2 - a2 * c1) / det])
}

pub fn segment_intersection(a: Point, b: Point, c: Point, d: Point) -> Option<Point> {
    let denominator = (d[1] - c[1]) * (b[0] - a[0]) - (d[0] - c[0]) * (b[1] - a[1]);
    if denominator.abs() < f64::EPSILON {
        return None;
    }

    let ua = ((d[0] - c[0]) * (a[1] - c[1]) - (d[1] - c[1]) * (a[0] - c[0])) / denominator;
    let ub = ((b[0] - a[0]) * (a[1] - c[1]) - (b[1] - a[1]) * (a[0] - c[0])) / denominator;

    if (-f64::EPSILON..=1.0 + f64::EPSILON).contains(&ua)
        && (-f64::EPSILON..=1.0 + f64::EPSILON).contains(&ub)
    {
        Some([a[0] + ua * (b[0] - a[0]), a[1] + ua * (b[1] - a[1])])
    } else {
        None
    }
}

pub fn polygon_edges(points: &[Point], close: bool) -> Vec<Line> {
    match points.len() {
        0 | 1 => Vec::new(),
        len => {
            let mut lines = Vec::with_capacity(if close { len } else { len - 1 });
            for pair in points.windows(2) {
                lines.push([pair[0], pair[1]]);
            }
            if close {
                lines.push([points[len - 1], points[0]]);
            }
            lines
        }
    }
}

pub fn bounding_box(points: &[Point]) -> Option<Rectangle> {
    let first = points.first()?;
    let (mut min_x, mut min_y) = (first[0], first[1]);
    let (mut max_x, mut max_y) = (first[0], first[1]);

    for point in &points[1..] {
        min_x = min_x.min(point[0]);
        min_y = min_y.min(point[1]);
        max_x = max_x.max(point[0]);
        max_y = max_y.max(point[1]);
    }

    Some(Rectangle {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

pub fn polygon_list_bounding_box(polygon_list: &[Vec<Point>]) -> Option<Rectangle> {
    let mut points = polygon_list.iter().flatten().copied();
    let first = points.next()?;
    let (mut min_x, mut min_y) = (first[0], first[1]);
    let (mut max_x, mut max_y) = (first[0], first[1]);

    for point in points {
        min_x = min_x.min(point[0]);
        min_y = min_y.min(point[1]);
        max_x = max_x.max(point[0]);
        max_y = max_y.max(point[1]);
    }

    Some(Rectangle {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    })
}

pub fn polygon_list_center(polygon_list: &[Vec<Point>]) -> Option<Point> {
    let mut count = 0.0;
    let mut sum = [0.0, 0.0];

    for point in polygon_list.iter().flatten() {
        sum[0] += point[0];
        sum[1] += point[1];
        count += 1.0;
    }

    if count == 0.0 {
        None
    } else {
        Some([sum[0] / count, sum[1] / count])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn line_length_uses_euclidean_distance() {
        assert_eq!(line_length([[0.0, 0.0], [3.0, 4.0]]), 5.0);
    }

    #[test]
    fn rotate_points_around_center() {
        let rotated = rotate_point([1.0, 0.0], [0.0, 0.0], std::f64::consts::FRAC_PI_2);

        assert_relative_eq!(rotated[0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(rotated[1], 1.0, epsilon = 1e-12);

        let points = rotate_points(&[[2.0, 1.0], [1.0, 2.0]], [1.0, 1.0], std::f64::consts::PI);
        assert_relative_eq!(points[0][0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(points[0][1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(points[1][0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(points[1][1], 0.0, epsilon = 1e-12);
    }

    #[test]
    fn rotate_lines_rotates_both_endpoints() {
        let lines = rotate_lines(
            &[[[1.0, 0.0], [2.0, 0.0]]],
            [0.0, 0.0],
            std::f64::consts::FRAC_PI_2,
        );

        assert_relative_eq!(lines[0][0][0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(lines[0][0][1], 1.0, epsilon = 1e-12);
        assert_relative_eq!(lines[0][1][0], 0.0, epsilon = 1e-12);
        assert_relative_eq!(lines[0][1][1], 2.0, epsilon = 1e-12);
    }

    #[test]
    fn line_intersection_handles_crossing_and_parallel_lines() {
        let crossing = line_intersection([0.0, 0.0], [2.0, 2.0], [0.0, 2.0], [2.0, 0.0])
            .expect("diagonals should cross");
        assert_relative_eq!(crossing[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(crossing[1], 1.0, epsilon = 1e-12);

        assert!(line_intersection([0.0, 0.0], [1.0, 0.0], [0.0, 2.0], [1.0, 2.0]).is_none());
    }

    #[test]
    fn segment_intersection_requires_overlap_on_both_segments() {
        let crossing = segment_intersection([0.0, 0.0], [2.0, 0.0], [1.0, -1.0], [1.0, 1.0])
            .expect("segments should cross");
        assert_relative_eq!(crossing[0], 1.0, epsilon = 1e-12);
        assert_relative_eq!(crossing[1], 0.0, epsilon = 1e-12);

        assert!(segment_intersection([0.0, 0.0], [1.0, 0.0], [2.0, -1.0], [2.0, 1.0]).is_none());
    }

    #[test]
    fn polygon_edges_can_close_path() {
        let points = [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0]];

        assert_eq!(polygon_edges(&points, false).len(), 2);
        assert_eq!(polygon_edges(&points, true).len(), 3);
    }

    #[test]
    fn bounding_boxes_cover_points_and_polygon_lists() {
        let points = [[2.0, 3.0], [-1.0, 5.0], [4.0, -2.0]];
        let bbox = bounding_box(&points).unwrap();

        assert_eq!(bbox.x, -1.0);
        assert_eq!(bbox.y, -2.0);
        assert_eq!(bbox.width, 5.0);
        assert_eq!(bbox.height, 7.0);

        let list_bbox = polygon_list_bounding_box(&[points.to_vec(), vec![[10.0, 1.0]]]).unwrap();
        assert_eq!(list_bbox.x, -1.0);
        assert_eq!(list_bbox.width, 11.0);
    }

    #[test]
    fn polygon_list_center_averages_all_points() {
        let center =
            polygon_list_center(&[vec![[0.0, 0.0], [2.0, 0.0]], vec![[4.0, 3.0]]]).unwrap();

        assert_relative_eq!(center[0], 2.0, epsilon = 1e-12);
        assert_relative_eq!(center[1], 1.0, epsilon = 1e-12);
        assert!(polygon_list_center(&[]).is_none());
    }
}
