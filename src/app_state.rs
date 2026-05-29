use crate::cache::Cache;
use crate::domain::settings::Settings;
use crate::parser::Parser;
use dashmap::DashMap;
use std::sync::Arc;

pub struct AppStateInner {
    pub settings: parking_lot::RwLock<Settings>,
    // Store parsed book structures in memory
    pub books: DashMap<String, crate::domain::book::Book>, 
    // Store parser instances
    pub parsers: DashMap<String, Arc<dyn Parser + Send + Sync>>,
    // Cache for rendered HTML and bionic text
    pub render_cache: Cache, 
    // Persistent library storage
    pub library: parking_lot::RwLock<crate::library::Library>,
}

pub type AppState = Arc<AppStateInner>;

impl AppStateInner {
    pub fn new() -> Self {
        let library = crate::library::Library::load();
        let books = DashMap::new();
        
        for (id, entry) in &library.entries {
            books.insert(id.clone(), entry.book.clone());
        }
        
        Self {
            settings: parking_lot::RwLock::new(Settings::default()),
            books,
            parsers: DashMap::new(),
            render_cache: Cache::new(),
            library: parking_lot::RwLock::new(library),
        }
    }
}
