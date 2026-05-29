use super::Parser;
use crate::domain::book::{Book, ChapterMeta};
use crate::error::AppError;
use epub::doc::EpubDoc;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

pub struct EpubParser {
    doc: Mutex<EpubDoc<std::io::BufReader<std::fs::File>>>,
}

impl Parser for EpubParser {
    fn parse_book<P: AsRef<Path>>(path: P) -> Result<Book, AppError> {
        let doc = EpubDoc::new(path.as_ref()).map_err(|e| anyhow::anyhow!("Failed to open EPUB: {}", e))?;
        
        let title = doc.mdata("title").map(|m| m.value.clone()).unwrap_or_else(|| "Unknown Title".to_string());
        let author = doc.mdata("creator").map(|m| m.value.clone());
        let _cover_image_path = doc.get_cover_id();

        let mut chapters = Vec::new();
        let spine = doc.spine.clone();
        
        for (order, spine_item) in spine.into_iter().enumerate() {
            let spine_id = spine_item.idref;
            chapters.push(ChapterMeta {
                id: spine_id.clone(),
                title: None, // Could parse TOC for titles later
                order,
                source_path: spine_id,
            });
        }
        
        // Use hash or Uuid for book id. We will use a deterministic hash of the path or title for now, or just a new uuid.
        let book_id = Uuid::new_v4().to_string();

        Ok(Book {
            id: book_id,
            title,
            author,
            cover_image_path: None, // Skip cover for now to keep simple
            chapters,
        })
    }

    fn extract_chapter_html(&self, chapter_id: &str) -> Result<String, AppError> {
        let mut doc = self.doc.lock().unwrap();
        
        let chapter_idx = doc.resource_id_to_chapter(chapter_id).ok_or_else(|| anyhow::anyhow!("Chapter ID not found in spine"))?;
        doc.set_current_chapter(chapter_idx);
        
        let (bytes, _mime) = doc.get_current().ok_or_else(|| anyhow::anyhow!("Failed to get page content"))?;
        let content = String::from_utf8(bytes).map_err(|e| anyhow::anyhow!("Invalid UTF-8 in chapter: {}", e))?;
        Ok(content)
    }

    fn extract_resource(&self, path: &str) -> Result<Vec<u8>, AppError> {
        let mut doc = self.doc.lock().unwrap();
        let (bytes, _mime) = doc.get_resource(path).ok_or_else(|| anyhow::anyhow!("Failed to get resource"))?;
        Ok(bytes)
    }
}

impl EpubParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let doc = EpubDoc::new(path.as_ref()).map_err(|e| anyhow::anyhow!("Failed to open EPUB: {}", e))?;
        Ok(Self {
            doc: Mutex::new(doc)
        })
    }
}
