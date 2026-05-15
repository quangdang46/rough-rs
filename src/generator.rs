use crate::core::{Config, Drawable, OpSet, Options, ResolvedOptions, ShapeType};
use crate::math::random_seed;
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
        let outline = renderer::rectangle(x, y, width, height, &resolved);
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

    fn drawable(&self, shape: ShapeType, sets: Vec<OpSet>, options: ResolvedOptions) -> Drawable {
        Drawable {
            shape,
            options,
            sets,
        }
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
