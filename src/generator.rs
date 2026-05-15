use crate::core::{Config, Drawable, Options, ResolvedOptions, ShapeType};
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
}

impl Default for Generator {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
