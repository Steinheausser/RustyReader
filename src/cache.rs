use dashmap::DashMap;

pub struct Cache {
    // Stores transformed chapter HTML fragments
    // Key format could be "{chapter_id}_{bionic_settings_hash}"
    rendered_chapters: DashMap<String, String>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            rendered_chapters: DashMap::new(),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<String> {
        self.rendered_chapters.get(key).map(|v| v.clone())
    }
    
    pub fn insert(&self, key: String, value: String) {
        self.rendered_chapters.insert(key, value);
    }
}
