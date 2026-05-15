use crate::core::{Config, Drawable, ResolvedOptions, ShapeType};
use crate::math::random_seed;
use crate::renderer;

#[derive(Debug, Clone)]
pub struct Generator {
    default_options: ResolvedOptions,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        let _ = config;
        Self {
            default_options: ResolvedOptions::default(),
        }
    }

    pub fn new_seed() -> u64 {
        random_seed()
    }

    pub fn default_options(&self) -> &ResolvedOptions {
        &self.default_options
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
