use ammonia::Builder;


/// Normalizes HTML from an EPUB chapter
pub fn normalize_html(raw_html: &str, book_id: &str) -> String {
    // We want to keep most standard reading tags, but remove scripts and styles.
    // Ammonia default is fairly safe. We'll customize it just a bit.
    let mut builder = Builder::default();
    
    // We need to rewrite image source URLs to our resource endpoint
    let url_prefix = format!("/api/resource/{}/", book_id);
    
    // Use `url_relative` to rewrite relative URLs
    builder.url_relative(ammonia::UrlRelative::RewriteWithBase(
        url::Url::parse(&format!("http://localhost{}", url_prefix)).unwrap()
    ));

    builder.clean(raw_html).to_string()
}
