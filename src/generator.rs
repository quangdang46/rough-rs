use crate::core::{Config, Drawable, OpSet, Options, ResolvedOptions, ShapeType};
use crate::geometry::Point;
use crate::math::{random_seed, RngHelper};
use crate::renderer;

#[derive(Debug, Clone)]
pub struct Generator {
    default_options: ResolvedOptions,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        let default_options = config
            .options
            .as_ref()
            .map(ResolvedOptions::from_options)
            .unwrap_or_default();
        Self { default_options }
    }

    pub fn new_seed() -> u64 {
        random_seed()
    }

    pub fn default_options(&self) -> &ResolvedOptions {
        &self.default_options
    }

    pub fn resolve_options(&self, options: Option<&Options>) -> ResolvedOptions {
        options
            .map(|options| self.default_options.clone().merge(options))
            .unwrap_or_else(|| self.default_options.clone())
    }

    pub fn empty(&self, shape: ShapeType) -> Drawable {
        Drawable {
            shape,
            options: self.default_options.clone(),
            sets: vec![renderer::empty_path(&self.default_options)],
        }
    }

    pub fn line(&self, x1: f64, y1: f64, x2: f64, y2: f64, options: Option<Options>) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        self.drawable(
            ShapeType::Line,
            vec![renderer::line(x1, y1, x2, y2, &resolved)],
            resolved,
        )
    }

    pub fn rectangle(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        options: Option<Options>,
    ) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        let mut sets = Vec::new();
        let mut rng = RngHelper::new(resolved.seed);
        let outline = renderer::rectangle_with_rng(x, y, width, height, &resolved, &mut rng);
        if resolved.fill.is_some() {
            let points = vec![
                [x, y],
                [x + width, y],
                [x + width, y + height],
                [x, y + height],
            ];
            sets.push(renderer::pattern_fill_polygons(
                &[points],
                &resolved,
                &mut rng,
            ));
        }
        if resolved.stroke != "none" {
            sets.push(outline);
        }
        self.drawable(ShapeType::Rectangle, sets, resolved)
    }

    pub fn ellipse(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        options: Option<Options>,
    ) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        let mut sets = Vec::new();
        let outline = renderer::ellipse(x, y, width, height, &resolved);
        if resolved.stroke != "none" {
            sets.push(outline);
        }
        self.drawable(ShapeType::Ellipse, sets, resolved)
    }

    pub fn circle(&self, x: f64, y: f64, diameter: f64, options: Option<Options>) -> Drawable {
        let mut drawable = self.ellipse(x, y, diameter, diameter, options);
        drawable.shape = ShapeType::Circle;
        drawable
    }

    pub fn linear_path(&self, points: &[Point], options: Option<Options>) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        self.drawable(
            ShapeType::LinearPath,
            vec![renderer::linear_path(points, false, &resolved)],
            resolved,
        )
    }

    pub fn polygon(&self, points: &[Point], options: Option<Options>) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        let mut rng = RngHelper::new(resolved.seed);
        let outline = renderer::linear_path_with_rng(points, true, &resolved, &mut rng);
        let mut sets = Vec::new();
        if resolved.fill.is_some() {
            sets.push(renderer::pattern_fill_polygons(
                &[points.to_vec()],
                &resolved,
                &mut rng,
            ));
        }
        if resolved.stroke != "none" {
            sets.push(outline);
        }
        self.drawable(ShapeType::Polygon, sets, resolved)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn arc(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        start: f64,
        stop: f64,
        closed: bool,
        options: Option<Options>,
    ) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        let mut rng = RngHelper::new(resolved.seed);
        let outline = renderer::arc_with_rng(
            x, y, width, height, start, stop, closed, true, &resolved, &mut rng,
        );
        let mut sets = Vec::new();
        if closed && resolved.fill.is_some() {
            if resolved.fill_style == crate::core::FillStyle::Solid {
                let mut fill_options = resolved.clone();
                fill_options.disable_multi_stroke = true;
                let mut shape = renderer::arc_with_rng(
                    x,
                    y,
                    width,
                    height,
                    start,
                    stop,
                    true,
                    false,
                    &fill_options,
                    &mut rng,
                );
                shape.set_type = crate::core::OpSetType::FillPath;
                sets.push(shape);
            } else {
                sets.push(renderer::pattern_fill_arc(
                    x, y, width, height, start, stop, &resolved, &mut rng,
                ));
            }
        }
        if resolved.stroke != "none" {
            sets.push(outline);
        }
        self.drawable(ShapeType::Arc, sets, resolved)
    }

    pub fn curve(&self, points: &[Point], options: Option<Options>) -> Drawable {
        let resolved = self.resolve_options(options.as_ref());
        let mut rng = RngHelper::new(resolved.seed);
        let outline = renderer::curve_with_rng(points, &resolved, &mut rng);
        let mut sets = Vec::new();
        if resolved.fill.as_deref().is_some_and(|fill| fill != "none") {
            if resolved.fill_style == crate::core::FillStyle::Solid {
                let mut fill_options = resolved.clone();
                fill_options.disable_multi_stroke = true;
                fill_options.roughness = if resolved.roughness != 0.0 {
                    resolved.roughness + resolved.fill_shape_roughness_gain
                } else {
                    0.0
                };
                let fill_shape = renderer::curve_with_rng(points, &fill_options, &mut rng);
                let ops = fill_shape
                    .ops
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, op)| {
                        if index == 0 || op.op != crate::core::OpType::Move {
                            Some(op)
                        } else {
                            None
                        }
                    })
                    .collect();
                sets.push(crate::core::OpSet::new(
                    crate::core::OpSetType::FillPath,
                    ops,
                ));
            } else {
                let poly_points = curve_fill_points(points, &resolved);
                if !poly_points.is_empty() {
                    sets.push(renderer::pattern_fill_polygons(
                        &[poly_points],
                        &resolved,
                        &mut rng,
                    ));
                }
            }
        }
        if resolved.stroke != "none" {
            sets.push(outline);
        }
        self.drawable(ShapeType::Curve, sets, resolved)
    }

    fn drawable(&self, shape: ShapeType, sets: Vec<OpSet>, options: ResolvedOptions) -> Drawable {
        Drawable {
            shape,
            options,
            sets,
        }
    }
}

fn curve_fill_points(points: &[Point], options: &ResolvedOptions) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let bezier_input = if points.len() == 3 {
        vec![points[0], points[0], points[1], points[2]]
    } else {
        points.to_vec()
    };
    renderer::curve_to_bezier(&bezier_input, options.curve_tightness)
        .map(|bezier| {
            renderer::points_on_bezier_curves(&bezier, 10.0, Some((1.0 + options.roughness) / 2.0))
        })
        .unwrap_or_default()
}

impl Default for Generator {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
