# tg-dl-bot -- Plan & Setup Guide

## Overview

A Rust Telegram bot that listens for messages containing links from YouTube, TikTok, Instagram, and Facebook, downloads the media (video, photo, or carousel), and sends it back to the user.

---

## Table of Contents

- [Tech Stack](#tech-stack)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [Configuration](#configuration)
- [Running](#running)
- [Docker Deployment](#docker-deployment)
- [Design Decisions](#design-decisions)
- [Supported Platforms](#supported-platforms)
- [Bot Commands](#bot-commands)
- [Telegram File Size Limits](#telegram-file-size-limits)
- [Future Improvements](#future-improvements)

---

## Tech Stack

| Layer          | Choice                           | Rationale                                                              |
| -------------- | -------------------------------- | ---------------------------------------------------------------------- |
| Language       | Rust 2021 edition                | Performance, safety, zero-cost abstractions                            |
| Bot framework  | `teloxide` v0.17                 | Most mature Rust Telegram SDK, strongly-typed, async, active community |
| Async runtime  | `tokio` (multi-threaded)         | Industry standard for async Rust                                       |
| Media download | `yt-dlp` CLI via subprocess      | Supports 1800+ sites, actively maintained against platform changes     |
| HTTP client    | `reqwest`                        | Built into teloxide; available for auxiliary HTTP work                 |
| Error handling | `thiserror` + `anyhow`           | Typed errors within modules, `anyhow` at application boundary          |
| Config         | `dotenvy` + `std::env`           | Simple `.env` file loading, no heavy framework                         |
| Logging        | `tracing` + `tracing-subscriber` | Structured, async-friendly, filterable by level                        |
| Serialization  | `serde` / `serde_json`           | Parse yt-dlp JSON output                                               |
| URL extraction | `linkify`                        | Robust URL detection in free-form message text                         |
| Rate limiting  | `dashmap`                        | Lock-free concurrent hashmap for per-user tracking                     |
| External tools | `yt-dlp`, `ffmpeg`               | Required at runtime for downloading and transcoding                    |

---

## Architecture

```
User sends message with link
        |
        v
+-------------------+
|  Telegram API      |  (long polling)
+-------------------+
        |
        v
+-------------------+
|  Dispatcher        |  teloxide dptree-based routing
+-------------------+
   |            |
   v            v
/start       text message
/help            |
                 v
        +------------------+
        |  URL Extraction  |  util/url.rs (linkify)
        +------------------+
                 |
                 v
        +------------------+
        | Platform Matcher |  platform/matcher.rs (regex)
        +------------------+
                 |
                 v
        +------------------+
        |  YtDlp Downloader|  downloader/ytdlp.rs (tokio::process)
        +------------------+
                 |
                 v
        +------------------+
        |  Media Sender    |  sender/upload.rs (size-aware)
        +------------------+
                 |
                 v
        +------------------+
        |  Temp Cleanup    |  util/cleanup.rs (Drop guard)
        +------------------+
```

**Data flow:** Message -> extract URLs -> classify platform -> download via yt-dlp -> upload to Telegram -> clean up temp files.

Each download runs as a spawned `tokio` task. A `Semaphore` caps concurrent downloads (default 4). A `DashMap<UserId, Instant>` enforces per-user rate limiting.

---

## Project Structure

```
tg-dl-bot/
├── Cargo.toml              # Dependencies and package metadata
├── Cargo.lock              # Locked dependency versions
├── .env.example            # Template for environment variables
├── .gitignore              # Ignores target/, .env, cookies, temp files
├── README.md               # User-facing documentation
├── PLAN.md                 # This file
├── Dockerfile              # Multi-stage build for production
└── src/
    ├── main.rs             # Entry: load config, init tracing, run dispatcher
    ├── config.rs           # Typed config from environment variables
    ├── error.rs            # App-wide error types (thiserror)
    ├── bot/
    │   ├── mod.rs          # Dispatcher setup, dependency injection
    │   ├── commands.rs     # /start and /help command handlers
    │   └── handlers.rs     # Message handler: link detection, orchestration,
    │                       #   rate limiting, progress feedback
    ├── downloader/
    │   ├── mod.rs          # Re-exports
    │   ├── types.rs        # MediaResult, MediaItem, MediaType
    │   └── ytdlp.rs        # yt-dlp subprocess wrapper with per-platform
    │                       #   format args and JSON metadata parsing
    ├── platform/
    │   ├── mod.rs          # Re-exports
    │   └── matcher.rs      # URL -> Platform enum via regex
    ├── sender/
    │   ├── mod.rs          # Re-exports
    │   └── upload.rs       # Telegram upload with file-size fallback logic
    └── util/
        ├── mod.rs          # Re-exports
        ├── url.rs          # URL extraction from message text (linkify)
        └── cleanup.rs      # TempDirGuard with Drop-based cleanup
```

### Module responsibilities

| Module        | Responsibility                         | Swap impact                                               |
| ------------- | -------------------------------------- | --------------------------------------------------------- |
| `bot/`        | Telegram-specific handlers and routing | Change here only if swapping bot framework                |
| `downloader/` | Media acquisition from URLs            | Swap yt-dlp for another backend without touching handlers |
| `platform/`   | URL classification                     | Add new platform = add one regex + match arm              |
| `sender/`     | Telegram upload logic and size checks  | Isolated from download logic                              |
| `util/`       | Small reusable helpers                 | Tree-shakable: unused utils are dead-code-eliminated      |

---

## Prerequisites

1. **Rust** 1.75+ -- install via [rustup](https://rustup.rs/)
2. **yt-dlp** -- install via `pip install yt-dlp` or your package manager
3. **ffmpeg** -- install via `brew install ffmpeg` (macOS) or `apt install ffmpeg` (Debian/Ubuntu)
4. **Telegram bot token** -- create one via [@BotFather](https://t.me/BotFather)

Verify tools are available:

```bash
rustc --version      # 1.75+
yt-dlp --version     # any recent version
ffmpeg -version      # any recent version
```

---

## Setup

### 1. Clone the repository

```bash
git clone <repo-url>
cd tg-dl-bot
```

### 2. Create environment file

```bash
cp .env.example .env
```

### 3. Set your bot token

Open `.env` and replace the placeholder:

```
TELOXIDE_TOKEN=123456789:ABCdefGHIjklMNOpqrSTUvwxYZ
```

### 4. Build the project

```bash
cargo build
```

### 5. Run

```bash
cargo run
```

The bot will start polling Telegram for updates. Send it a YouTube/TikTok/Instagram/Facebook link to test.

---

## Configuration

All settings are via environment variables (or `.env` file):

| Variable                   | Default          | Description                                                |
| -------------------------- | ---------------- | ---------------------------------------------------------- |
| `TELOXIDE_TOKEN`           | **required**     | Telegram bot token from BotFather                          |
| `MAX_CONCURRENT_DOWNLOADS` | `4`              | Maximum parallel yt-dlp downloads                          |
| `RATE_LIMIT_SECONDS`       | `10`             | Per-user cooldown between download requests                |
| `COOKIES_PATH`             | none             | Path to a Netscape-format `cookies.txt` for Instagram auth |
| `RUST_LOG`                 | `tg_dl_bot=info` | Log level filter (`debug`, `info`, `warn`, `error`)        |

---

## Running

### Development

```bash
# With debug logging
RUST_LOG=tg_dl_bot=debug cargo run
```

### Release build

```bash
cargo build --release
./target/release/tg-dl-bot
```

### Run tests

```bash
cargo test
```

---

## Docker Deployment

### Build the image

```bash
docker build -t tg-dl-bot .
```

The multi-stage Dockerfile:

1. **Build stage** -- compiles the Rust binary in `rust:1.85-slim`
2. **Runtime stage** -- copies the binary into `debian:bookworm-slim` with `yt-dlp` and `ffmpeg` installed

### Run the container

```bash
docker run -d \
  --name tg-dl-bot \
  --env-file .env \
  --restart unless-stopped \
  tg-dl-bot
```

### With Instagram cookies

Mount a cookies file into the container:

```bash
docker run -d \
  --name tg-dl-bot \
  --env-file .env \
  -e COOKIES_PATH=/data/cookies.txt \
  -v /path/to/cookies.txt:/data/cookies.txt:ro \
  --restart unless-stopped \
  tg-dl-bot
```

---

## Design Decisions

### 1. yt-dlp via subprocess, not native Rust

No single Rust crate supports YouTube + TikTok + Instagram + Facebook. yt-dlp covers 1800+ sites and is actively maintained against platform API changes. We call it via `tokio::process::Command` with `--print-json` for metadata and `-o` for file output.

### 2. Trait-based downloader

The `YtDlpDownloader` can be swapped or mocked without changing any handler code. Adding a new download backend means implementing the same interface.

### 3. Per-platform format selectors

| Platform  | yt-dlp `-f` flag                                                         | Rationale                                              |
| --------- | ------------------------------------------------------------------------ | ------------------------------------------------------ |
| YouTube   | `bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best` | Caps at 1080p mp4 to stay under Telegram's 50 MB limit |
| TikTok    | `best[ext=mp4]/best`                                                     | TikTok videos are typically small                      |
| Instagram | `best`                                                                   | Reels, posts, stories -- varied formats                |
| Facebook  | `best[ext=mp4]/best`                                                     | Prefer mp4 container                                   |

### 4. Size-aware upload strategy

| Media type | Size     | Telegram method |
| ---------- | -------- | --------------- |
| Photo      | <= 10 MB | `sendPhoto`     |
| Photo      | > 10 MB  | `sendDocument`  |
| Video      | <= 50 MB | `sendVideo`     |
| Video      | > 50 MB  | `sendDocument`  |

### 5. Temp file lifecycle

Each download gets its own `tempfile::tempdir()`. A `TempDirGuard` wraps the path and implements `Drop` -- when the guard goes out of scope (after upload or on error), the directory is removed. This prevents disk leaks even on panics.

### 6. Concurrency control

- **Download semaphore** -- `tokio::sync::Semaphore` with `MAX_CONCURRENT_DOWNLOADS` permits. Prevents CPU/bandwidth exhaustion.
- **Per-user rate limit** -- `DashMap<UserId, Instant>` tracks last request time. Users exceeding the cooldown get a friendly "please wait" message.

### 7. Progress feedback

The bot sends status messages to keep users informed:

1. "Downloading from YouTube..." (sent immediately, as a reply)
2. "Uploading 1 file(s)..." (edited in-place after download completes)
3. Status message deleted after media is delivered

---

## Supported Platforms

| Platform  | Videos | Photos  | Carousel | Notes                               |
| --------- | ------ | ------- | -------- | ----------------------------------- |
| YouTube   | yes    | --      | --       | Videos and Shorts                   |
| TikTok    | yes    | yes     | --       | Standard posts                      |
| Instagram | yes    | partial | partial  | Full carousel support needs cookies |
| Facebook  | yes    | yes     | --       | Posts, Reels, Stories               |

---

## Bot Commands

| Command  | Description                                   |
| -------- | --------------------------------------------- |
| `/start` | Welcome message explaining how to use the bot |
| `/help`  | Lists supported platforms and usage notes     |

All other messages are scanned for supported URLs. Non-matching messages are silently ignored.

---

## Telegram File Size Limits

The standard Telegram Bot API enforces these limits:

| Method         | Max size |
| -------------- | -------- |
| `sendPhoto`    | 10 MB    |
| `sendVideo`    | 50 MB    |
| `sendDocument` | 50 MB    |

Files exceeding the `sendVideo` / `sendPhoto` limits are automatically sent as documents. Files exceeding 50 MB entirely would need a local Bot API server or chunked upload (not yet implemented).

---

## Future Improvements

- **Configurable quality** -- inline keyboard letting users pick 360p / 720p / 1080p / best
- **Admin `/status` command** -- uptime, download count, queue depth
- **Retry logic** -- automatic retry (up to 2 times with backoff) on transient yt-dlp failures
- **Request ID tracing** -- `tracing` spans keyed by `update_id` for end-to-end debuggability
- **Chunked upload** -- for files > 50 MB via Telegram's `saveBigFilePart` API
- **Local Bot API server** -- removes the 50 MB upload cap entirely
- **Webhook mode** -- switch from long polling to webhooks for production traffic
- **Database persistence** -- track download history, user stats, popular URLs
- **Audio extraction** -- `/audio` command to extract audio-only from videos
