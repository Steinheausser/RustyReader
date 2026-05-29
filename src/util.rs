// Utility functions e.g. for hashing, file system operations, etc.

pub fn generate_cache_key(chapter_id: &str, bionic_setting: u8) -> String {
    format!("{}_{}", chapter_id, bionic_setting)
}
