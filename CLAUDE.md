# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```sh
cargo build          # Build debug version
cargo build --release # Build release version
cargo run -- <url>   # Run with a YouTube URL
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt            # Format code
```

## External Dependencies

This tool requires `yt-dlp` to be installed and available in PATH. Install via:
- macOS: `brew install yt-dlp`
- Linux: `pip install yt-dlp` or package manager

## Architecture

Single-binary CLI tool that:
1. Takes a YouTube URL as argument (via clap)
2. Calls `yt-dlp` to download and extract audio as MP3
3. Renames the output file to a sanitized lowercase filename (alphanumeric + dashes only)

The codebase is a single `main.rs` with:
- `Args` struct for CLI parsing
- `sanitize_filename()` for cleaning output names
- `download_and_extract_mp3()` async function that orchestrates the download and rename
