pub mod epub;

use crate::domain::book::Book;
use crate::error::AppError;
use std::path::Path;

pub trait Parser {
    fn parse_book<P: AsRef<Path>>(path: P) -> Result<Book, AppError> where Self: Sized;
    fn extract_chapter_html(&self, chapter_id: &str) -> Result<String, AppError>;
    fn extract_resource(&self, path: &str) -> Result<Vec<u8>, AppError>;
}
