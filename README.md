# Fast Speed Reader

A fast, local-first e-reader built in Rust. It serves books via an embedded HTTP server and opens a lightweight, snappy reading UI in your default browser. Features an integrated Bionic Reading mode for improved reading speed.

## Quickstart

```bash
# Build the project
cargo build --release

# Open a book (starts server and opens browser automatically)
./target/release/fast-ereader open path/to/book.epub

# Run in background without opening browser
./target/release/fast-ereader open path/to/book.epub --no-browser --background
```

## Architecture Notes
- Built on `axum` and `tokio`.
- UI is entirely web-based (HTML/JS/CSS) avoiding heavy native GUI toolkits.
- EPUB chapters are normalized, optionally bionic-transformed, and cached on the server for instant page loads.
