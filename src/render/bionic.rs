use crate::domain::settings::BionicSettings;
use scraper::Html;

/// Transforms a normalized HTML string into its bionic reading equivalent.
pub fn apply_bionic_reading(html: &str, settings: &BionicSettings) -> String {
    let _document = Html::parse_fragment(html);
    
    // For a real implementation, we'd need to walk the DOM tree, mutating text nodes.
    // `scraper` doesn't provide easy mutable DOM tree modification. 
    // We might have to build the HTML string manually by traversing the tree, or use regex as a fallback.
    // A robust way in Rust without a mutable DOM is `lol_html` or regex tokenization if the HTML is already sanitized.
    
    // As a placeholder for Phase 1, we will tokenize by words ignoring HTML tags, which is naive but works for a prototype.
    // For a true production app, we'd use `lol_html` for streaming rewrite, or parse DOM and serialize.
    // Let's implement a naive string replacement based on words.
    
    let mut result = String::with_capacity(html.len() * 2);
    let mut in_tag = false;
    let mut current_word = String::new();
    
    for c in html.chars() {
        if c == '<' {
            if !current_word.is_empty() {
                result.push_str(&process_word(&current_word, settings.intensity));
                current_word.clear();
            }
            in_tag = true;
            result.push(c);
        } else if c == '>' {
            in_tag = false;
            result.push(c);
        } else if in_tag {
            result.push(c);
        } else {
            if c.is_alphanumeric() {
                current_word.push(c);
            } else {
                if !current_word.is_empty() {
                    result.push_str(&process_word(&current_word, settings.intensity));
                    current_word.clear();
                }
                result.push(c);
            }
        }
    }
    
    if !current_word.is_empty() {
        result.push_str(&process_word(&current_word, settings.intensity));
    }
    
    result
}

/// Compute how many characters of a word should be bolded.
fn compute_prefix_len(word: &str, intensity: f32) -> usize {
    let len = word.chars().count();
    if len <= 3 {
        1
    } else if len <= 5 {
        2
    } else {
        (len as f32 * intensity).ceil() as usize
    }
}

fn process_word(word: &str, intensity: f32) -> String {
    let prefix_len = compute_prefix_len(word, intensity);
    let (prefix, suffix) = split_at_char_boundary(word, prefix_len);
    format!(r#"<span class="br-word"><span class="br-focus">{}</span>{}</span>"#, prefix, suffix)
}

fn split_at_char_boundary(s: &str, count: usize) -> (&str, &str) {
    let mut _byte_idx = 0;
    for (i, (idx, _)) in s.char_indices().enumerate() {
        if i == count {
            return s.split_at(idx);
        }
        _byte_idx = idx;
    }
    (s, "")
}
