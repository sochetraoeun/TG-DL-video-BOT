# tg-dl-bot

A Telegram bot that downloads videos and photos from YouTube, TikTok, Instagram, and Facebook, then sends them directly to your chat.

## Features

- Automatic link detection in messages (no commands needed)
- YouTube, TikTok, Instagram, and Facebook support
- Videos, photos, and carousel posts
- Size-aware uploads (video/photo/document depending on file size)
- Progress feedback (Downloading... → Uploading... → delivered)
- **Fallback download chain** — when yt-dlp fails:
  - **YouTube**: tries rusty_ytdl → rusty_dl
  - **TikTok/Instagram/Facebook**: tries gallery-dl (install with `pip install gallery-dl`)
- Per-user rate limiting
- Concurrent download control via semaphore
- Automatic temp file cleanup

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+ — install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) on `PATH` — `pip install yt-dlp` or download from releases
- [ffmpeg](https://ffmpeg.org/) on `PATH` — `brew install ffmpeg` (macOS) or your package manager
- [gallery-dl](https://github.com/mikf/gallery-dl) (optional) — fallback when yt-dlp fails: `pip install gallery-dl`
- A Telegram bot token from [@BotFather](https://t.me/BotFather)

## Installation & Run (after cloning)

```bash
# 1. Clone the repository
git clone <repo-url> && cd TG-DL-video-BOT

# 2. Install dependencies (Rust crates are built automatically)
cargo build

# 3. Configure environment
cp .env.example .env
# Edit .env and set your TELOXIDE_TOKEN (get it from @BotFather)

# 4. Run the bot
cargo run
```

To build an optimized release binary:

```bash
cargo build --release
./target/release/tg-dl-bot
```

## Configuration

All configuration is via environment variables (or `.env` file):

| Variable                   | Default          | Description                              |
| -------------------------- | ---------------- | ---------------------------------------- |
| `TELOXIDE_TOKEN`           | _required_       | Telegram bot token                       |
| `MAX_CONCURRENT_DOWNLOADS` | `4`              | Max parallel downloads                   |
| `RATE_LIMIT_SECONDS`       | `10`             | Per-user cooldown between downloads      |
| `COOKIES_PATH`             | _none_           | Path to `cookies.txt` for Instagram/auth |
| `RUST_LOG`                 | `tg_dl_bot=info` | Log level filter                         |

## Docker

```bash
docker build -t tg-dl-bot .
docker run -d --env-file .env --name tg-dl-bot tg-dl-bot
```

## Project Structure

3

```
src/
├── main.rs              # Entry point
├── config.rs            # Typed config from env
├── error.rs             # App-wide error types
├── bot/
│   ├── commands.rs      # /start, /help
│   └── handlers.rs      # Link detection, download orchestration
├── downloader/
│   ├── types.rs         # MediaResult, MediaItem, MediaType
│   └── ytdlp.rs        # yt-dlp subprocess wrapper
├── platform/
│   └── matcher.rs       # URL → Platform detection
├── sender/
│   └── upload.rs        # Telegram media upload with size handling
└── util/
    ├── url.rs           # URL extraction from message text
    └── cleanup.rs       # Temp directory guard (Drop-based)
```

## Bot Commands

| Command  | Description                        |
| -------- | ---------------------------------- |
| `/start` | Welcome message                    |
| `/help`  | Supported platforms and usage info |

## Group Support

The bot works in groups. Add it to any group and:

- **With privacy mode (default):** Mention the bot with a link, e.g. `@YourBotName https://youtube.com/watch?v=...`
- **Without privacy mode:** Paste links normally; the bot will process them automatically. To disable privacy mode: [@BotFather](https://t.me/BotFather) → Bot Settings → Group Privacy → Turn off

## How It Works

1. User sends a message containing a link
2. Bot extracts URLs and identifies the platform
3. Downloads media via `yt-dlp` (async subprocess)
4. Sends media to the user (video, photo, or document depending on size)
5. Cleans up temp files

## Telegram File Size Limits

| Type  | Limit | Fallback         |
| ----- | ----- | ---------------- |
| Photo | 10 MB | Sent as document |
| Video | 50 MB | Sent as document |

## License

MIT
