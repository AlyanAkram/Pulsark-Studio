use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
};

use eframe::egui::text::LayoutJob;

pub struct CachedHighlight {
    pub hash: u64,
    pub layout: LayoutJob,
}

pub struct HighlightCache {
    cache: HashMap<PathBuf, CachedHighlight>,
}

impl HighlightCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn calc_hash(text: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_or_insert<F>(
        &mut self,
        path: &PathBuf,
        text: &str,
        build: F,
    ) -> LayoutJob
    where
        F: FnOnce() -> LayoutJob,
    {
        let hash = Self::calc_hash(text);

        if let Some(existing) = self.cache.get(path) {
            if existing.hash == hash {
                return existing.layout.clone();
            }
        }

        let layout = build();

        self.cache.insert(
            path.clone(),
            CachedHighlight {
                hash,
                layout: layout.clone(),
            },
        );

        layout
    }
}