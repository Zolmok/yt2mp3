# yt2mp3

Download YouTube videos and extract audio as MP3.

## Requirements

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) must be installed and available in PATH
- [FFmpeg](https://ffmpeg.org/) (required by yt-dlp for audio extraction)

### Install dependencies

**macOS:**
```sh
brew install yt-dlp ffmpeg
```

**Linux:**
```sh
# Debian/Ubuntu
sudo apt install ffmpeg
pip install yt-dlp

# Arch
sudo pacman -S yt-dlp ffmpeg
```

## Installation

```sh
cargo install --path .
```

## Usage

```sh
yt2mp3 <youtube-url>
```

### Example

```sh
yt2mp3 "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
```

The MP3 will be saved in the current directory with a sanitized filename (lowercase, alphanumeric with dashes).

## Building from source

```sh
git clone https://github.com/yourusername/yt2mp3
cd yt2mp3
cargo build --release
```

The binary will be at `target/release/yt2mp3`.

## Running tests

```sh
cargo test
```

## License

MIT
