//! Incremental parse cache (owned by task 34).

use crate::ast::item::Item;
use std::sync::Arc;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
}

/// Placeholder cache; task 34 replaces the body, not this shape.
#[derive(Debug, Default)]
pub struct Cache {
    stats: CacheStats,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stats(&self) -> CacheStats {
        self.stats
    }

    pub fn clear(&mut self) {}

    pub fn get_or_parse(&mut self, _path: &str, _source: &str) -> Arc<[Item]> {
        self.stats.misses += 1;
        Arc::from(Vec::<Item>::new())
    }
}
