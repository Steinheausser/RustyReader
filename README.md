# RustyReader (Fast Speed Reader)

An RSVP (speed reading) Reader with primarily epub, chunking, and other support built with a Rust backend and web frontend.
Very heavily inspired by [LetoReader](https://github.com/Axym-Labs/LetoReader), except I wanted something that I could run natively without docker in Windows and with hopefully a lower memory profile. This is entirely vibe-coded / AI slop so take your necessary precautions.

It serves books via an embedded HTTP server and opens a UI in your default browser. Features an integrated Bionic Reading mode for improved reading speed.

## Quickstart

Make sure you have cargo and Rust installed on your system.

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
