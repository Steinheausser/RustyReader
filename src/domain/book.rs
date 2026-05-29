use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_image_path: Option<String>,
    pub chapters: Vec<ChapterMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterMeta {
    pub id: String,
    pub title: Option<String>,
    pub order: usize,
    pub source_path: String,
}
