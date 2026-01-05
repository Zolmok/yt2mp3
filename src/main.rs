use clap::Parser;
use regex::Regex;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

static RE_NON_ALNUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9]+").expect("valid regex"));
static RE_MULTI_DASH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-+").expect("valid regex"));

const OUTPUT_TEMPLATE: &str = "%(title)s.%(ext)s";

/// Download a YouTube video and extract MP3.
#[derive(Parser)]
struct Args {
    /// The YouTube video URL
    url: String,
}

fn sanitize_filename(title: &str) -> String {
    let no_quotes = title.replace('\'', "");
    let mut cleaned = RE_NON_ALNUM.replace_all(&no_quotes, "-").to_string();
    cleaned = RE_MULTI_DASH.replace_all(&cleaned, "-").to_string();
    let mut result = cleaned.trim_matches('-').to_lowercase();

    // Handle empty result (e.g., non-ASCII only titles) by generating a hash-based name
    if result.is_empty() {
        result = format!(
            "audio-{:x}",
            title
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_add(b as u64))
        );
    }

    // Truncate to stay within filesystem limits (200 bytes, leaving room for suffix + extension)
    if result.len() > 200 {
        result.truncate(200);
        result = result.trim_end_matches('-').to_string();
    }

    result
}

/// Move a file, falling back to copy+delete for cross-filesystem moves.
fn move_file(from: &Path, to: &Path) -> Result<(), String> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::CrossesDevices => {
            fs::copy(from, to).map_err(|e| format!("Copy failed: {}", e))?;
            fs::remove_file(from).map_err(|e| format!("Remove original failed: {}", e))?;
            Ok(())
        }
        Err(e) => Err(format!("Rename failed: {}", e)),
    }
}

/// Find a unique filename by appending a number suffix if needed.
fn unique_path(base: &str, ext: &str) -> std::path::PathBuf {
    let candidate = format!("{}.{}", base, ext);
    if !Path::new(&candidate).exists() {
        return candidate.into();
    }

    for i in 1..1000 {
        let candidate = format!("{}-{}.{}", base, i, ext);
        if !Path::new(&candidate).exists() {
            return candidate.into();
        }
    }
    // Fallback with process ID if somehow 1000 collisions exist
    format!("{}-{}.{}", base, std::process::id(), ext).into()
}

fn download_and_extract_mp3(url: &str) -> Result<(), String> {
    // Download with visible progress (stdout/stderr inherited)
    let status = Command::new("yt-dlp")
        .args(["-x", "--audio-format", "mp3", "-o", OUTPUT_TEMPLATE, url])
        .status()
        .map_err(|e| format!("Failed to start yt-dlp: {}", e))?;

    if !status.success() {
        return Err(format!("yt-dlp failed with status: {}", status));
    }

    // Query yt-dlp for the video title
    let output = Command::new("yt-dlp")
        .args(["--print", "%(title)s", "--no-download", url])
        .output()
        .map_err(|e| format!("Failed to get title: {}", e))?;

    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() {
        return Err("yt-dlp produced no title".to_string());
    }

    let downloaded_file = format!("{}.mp3", title);
    let downloaded_path = Path::new(&downloaded_file);

    if !downloaded_path.exists() {
        return Err(format!("Downloaded file not found: {}", downloaded_file));
    }

    let cleaned_name = sanitize_filename(&title);
    let new_path = unique_path(&cleaned_name, "mp3");

    if downloaded_path != new_path {
        move_file(downloaded_path, &new_path)?;
    }
    println!("Saved: {}", new_path.display());

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(err) = download_and_extract_mp3(&args.url) {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_basic() {
        assert_eq!(sanitize_filename("Hello World"), "hello-world");
        assert_eq!(sanitize_filename("Test 123"), "test-123");
    }

    #[test]
    fn test_sanitize_special_chars() {
        assert_eq!(sanitize_filename("Hello! @World#"), "hello-world");
        assert_eq!(sanitize_filename("foo---bar"), "foo-bar");
        assert_eq!(sanitize_filename("--trim--"), "trim");
    }

    #[test]
    fn test_sanitize_quotes() {
        assert_eq!(sanitize_filename("It's a test"), "its-a-test");
        assert_eq!(sanitize_filename("Rock 'n' Roll"), "rock-n-roll");
    }

    #[test]
    fn test_sanitize_non_ascii() {
        // Non-ASCII only should produce a hash-based fallback
        let result = sanitize_filename("日本語");
        assert!(result.starts_with("audio-"));
    }

    #[test]
    fn test_sanitize_empty() {
        // Empty string should produce a hash-based fallback
        let result = sanitize_filename("");
        assert!(result.starts_with("audio-"));
    }

    #[test]
    fn test_sanitize_long_filename() {
        let long_title = "a".repeat(300);
        let result = sanitize_filename(&long_title);
        assert!(result.len() <= 200);
    }

    #[test]
    fn test_unique_path_no_collision() {
        // With a random enough name, should return base.ext
        let unique_name = format!("test-unique-{}", std::process::id());
        let result = unique_path(&unique_name, "mp3");
        assert_eq!(result.to_string_lossy(), format!("{}.mp3", unique_name));
    }
}
