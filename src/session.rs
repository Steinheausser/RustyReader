// Session tracking, progress, bookmarks, etc.

#[allow(dead_code)]
pub struct Session {
    pub current_book_id: Option<String>,
    pub current_chapter_id: Option<String>,
    pub scroll_progress: f32,
}
