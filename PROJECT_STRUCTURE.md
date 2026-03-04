# tg-dl-bot — Project Structure & File Reference

This document explains the purpose of each folder and file in the **tg-dl-bot** project — a Telegram bot that downloads videos and photos from YouTube, TikTok, Instagram, and Facebook.

---

## Root Directory

| File             | Purpose                                                                                                                                        |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| **Cargo.toml**   | Rust package manifest. Defines project name, version, edition, and all dependencies (teloxide, tokio, serde, reqwest, linkify, dashmap, etc.). |
| **Cargo.lock**   | Locked dependency versions. Ensures reproducible builds across machines.                                                                       |
| **.env**         | Local environment variables (not committed). Contains `TELOXIDE_TOKEN`, rate limits, and optional `COOKIES_PATH`.                              |
| **.env.example** | Template for `.env`. Shows required and optional variables with defaults. Copy to `.env` and fill in your bot token.                           |
| **.gitignore**   | Tells Git to ignore `target/`, `.env`, `cookies.txt`, `*.tmp`, and `/downloads`.                                                               |
| **README.md**    | User-facing documentation: features, quick start, configuration, Docker usage.                                                                 |
| **PLAN.md**      | Internal design document: architecture, tech stack, design decisions, future improvements.                                                     |
| **Dockerfile**   | Multi-stage Docker build: Stage 1 compiles Rust binary; Stage 2 runs it on Debian with yt-dlp and ffmpeg installed.                            |

---

## `src/` — Main Application Code

### `src/main.rs`

**Purpose:** Application entry point.

- Loads `.env` via `dotenvy`
- Initializes `tracing` logging with env filter (`RUST_LOG`)
- Parses config from environment
- Starts the bot dispatcher via `bot::run(config)`

---

### `src/config.rs`

**Purpose:** Typed configuration from environment variables.

- **`Config`** struct holds:
  - `max_concurrent_downloads` — max parallel yt-dlp downloads (default: 4)
  - `rate_limit_seconds` — per-user cooldown between requests (default: 10)
  - `cookies_path` — optional path to `cookies.txt` for Instagram
- **`from_env()`** reads `MAX_CONCURRENT_DOWNLOADS`, `RATE_LIMIT_SECONDS`, `COOKIES_PATH` from env

---

### `src/error.rs`

**Purpose:** Application-wide error types using `thiserror`.

Defines **`AppError`** variants:

| Variant               | When used                                |
| --------------------- | ---------------------------------------- |
| `Download`            | Generic download failure                 |
| `YtDlpProcess`        | yt-dlp exits with non-zero code          |
| `YtDlpNotFound`       | yt-dlp not installed or not on PATH      |
| `MetadataParse`       | Failed to parse yt-dlp JSON output       |
| `UnsupportedPlatform` | URL doesn't match any supported platform |
| `FileTooLarge`        | File exceeds Telegram limits             |
| `Telegram`            | Telegram API error                       |
| `RateLimited`         | User hit rate limit                      |
| `NoMedia`             | No downloadable media found at URL       |
| `Io`                  | I/O error (from `std::io::Error`)        |
| `Json`                | JSON parse error (from `serde_json`)     |

---

## `src/bot/` — Telegram Bot Logic

Handles Telegram-specific routing, commands, and message processing.

### `src/bot/mod.rs`

**Purpose:** Bot setup and dispatcher wiring.

- Creates `Bot` from `TELOXIDE_TOKEN` (via env)
- Instantiates `YtDlpDownloader`, `MediaSender`, and `State` (shared across handlers)
- Builds **dptree** dispatcher:
  - **Branch 1:** Commands (`/start`, `/help`) → `commands::handle_command`
  - **Branch 2:** All other messages → `handlers::handle_message`
- Injects dependencies (`downloader`, `media_sender`, `state`)
- Registers `LogErrorHandler` for unhandled errors
- Enables Ctrl+C handler for graceful shutdown

---

### `src/bot/commands.rs`

**Purpose:** Command handlers for `/start` and `/help`.

- **`Command`** enum (via `BotCommands` derive): `Start`, `Help`
- **`handle_command()`** sends:
  - **`/start`** — Welcome message explaining how to use the bot
  - **`/help`** — Supported platforms (YouTube, TikTok, Instagram, Facebook), usage notes, file size limits

---

### `src/bot/handlers.rs`

**Purpose:** Main message handler — link detection, download orchestration, rate limiting.

- **`State`** struct:
  - `rate_limits` — `DashMap<UserId, Instant>` for per-user cooldown
  - `rate_limit_duration` — from config
  - `semaphore` — limits concurrent downloads
- **`handle_message()`**:
  1. Extracts text from message; returns if none
  2. Extracts supported URLs via `extract_supported_urls()`
  3. Checks rate limit; sends "please wait" if exceeded
  4. For each URL: sends "Downloading from {platform}..." status
  5. Spawns async task per URL (acquires semaphore, runs `process_download`)
- **`process_download()`**:
  1. Creates temp dir with `TempDirGuard` (auto-cleanup on drop)
  2. Calls `downloader.download()`
  3. Edits status to "Uploading N file(s)..."
  4. Sends each media item via `media_sender.send_media()`
  5. Deletes status message when done
- **`LogErrorHandler`** — logs unhandled dispatcher errors

---

## `src/downloader/` — Media Download Logic

Handles fetching media from URLs. Uses yt-dlp as primary; when it fails for YouTube, tries **rusty_ytdl** then **rusty_dl** as fallbacks. Status stays "Downloading..." during the entire fallback chain.

### `src/downloader/mod.rs`

**Purpose:** Re-exports `pipeline`, `rusty_ytdl_fallback`, `rusty_dl_fallback`, `types`, `ytdlp`.

### `src/downloader/pipeline.rs`

**Purpose:** Download pipeline with fallback chain.

- **`DownloadPipeline`** — wraps `YtDlpDownloader`
- **`download_with_fallback()`** — tries in order:
  1. **yt-dlp** (all platforms)
  2. **rusty_ytdl** (YouTube only, if yt-dlp fails)
  3. **rusty_dl** (YouTube only, if rusty_ytdl fails)
- Keeps "Downloading..." status during all attempts; only shows error after all fail

### `src/downloader/rusty_ytdl_fallback.rs`

**Purpose:** Fallback #1 for YouTube using pure-Rust `rusty_ytdl` crate.

- **`download_youtube()`** — downloads to output dir, returns `MediaResult`
- Used when yt-dlp fails (e.g. rate limit, version mismatch)

### `src/downloader/rusty_dl_fallback.rs`

**Purpose:** Fallback #2 for YouTube using `rusty_dl` crate.

- **`download_youtube()`** — downloads via `YoutubeDownloader`, scans output dir for media
- Used when both yt-dlp and rusty_ytdl fail

---

### `src/downloader/types.rs`

**Purpose:** Data structures for download results.

- **`MediaType`** — `Video` or `Photo`
- **`MediaItem`** — `path`, `media_type`, `size_bytes`
- **`MediaResult`** — `items` (Vec of MediaItem), `title`, `platform`, `source_url`

---

### `src/downloader/ytdlp.rs`

**Purpose:** yt-dlp subprocess wrapper.

- **`YtDlpDownloader`** — holds optional `cookies_path`
- **`download()`**:
  1. Ensures yt-dlp is available
  2. Builds `Command` with `--no-playlist`, `--write-info-json`, `--print-json`, `-o` template
  3. Applies platform-specific `-f` format args (YouTube: 1080p mp4, TikTok/Instagram/Facebook: best mp4)
  4. Optionally adds `--cookies` if configured
  5. Runs yt-dlp, parses JSON output
- **`parse_output()`** — parses JSON lines, collects `MediaItem`s from `requested_downloads` or `entries`; fallback: scans output dir for media files
- **`file_to_media_item()`** — maps file path to `MediaItem` (jpg/jpeg/png/webp → Photo, else → Video)
- **`ensure_ytdlp_available()`** — runs `yt-dlp --version` to check availability

---

## `src/platform/` — URL Platform Detection

Identifies which platform a URL belongs to.

### `src/platform/mod.rs`

**Purpose:** Re-exports `matcher` submodule.

---

### `src/platform/matcher.rs`

**Purpose:** URL → Platform classification via regex.

- **`Platform`** enum: `YouTube`, `TikTok`, `Instagram`, `Facebook`
- **`display_name()`** — human-readable name for status messages
- **`detect_platform(url)`** — returns `Some(Platform)` if URL matches one of:
  - YouTube: `youtube.com`, `youtu.be`, `m.youtube.com`
  - TikTok: `tiktok.com`, `vm.tiktok.com`
  - Instagram: `instagram.com`, `instagr.am`
  - Facebook: `facebook.com`, `fb.watch`, `fb.com`, `m.facebook.com`
- Contains unit tests for each platform and unknown URLs

---

## `src/sender/` — Telegram Upload Logic

Sends downloaded media to the user with size-aware method selection.

### `src/sender/mod.rs`

**Purpose:** Re-exports `upload` submodule.

---

### `src/sender/upload.rs`

**Purpose:** Upload media to Telegram with size fallbacks.

- **`MediaSender`** — holds config (for future use)
- **`send_media()`**:
  - **Photo:** ≤ 10 MB → `sendPhoto`; > 10 MB → `sendDocument`
  - **Video:** ≤ 50 MB → `sendVideo`; > 50 MB → `sendDocument`
- **`send_as_document()`** — helper for oversized files
- Uses `InputFile::file()` to attach local files

---

## `src/util/` — Shared Utilities

Reusable helpers used across the app.

### `src/util/mod.rs`

**Purpose:** Re-exports `cleanup` and `url` submodules.

---

### `src/util/url.rs`

**Purpose:** Extract supported URLs from message text.

- **`ExtractedUrl`** — `url` (String), `platform` (Platform)
- **`extract_supported_urls(text)`** — uses `linkify::LinkFinder` to find URLs, filters to only those matching a supported platform via `detect_platform()`
- Contains unit tests for single/multiple URLs, unsupported URLs, plain text

---

### `src/util/cleanup.rs`

**Purpose:** Temp directory cleanup guard.

- **`TempDirGuard`** — wraps a `PathBuf`, implements `Drop`
- On drop: removes directory via `std::fs::remove_dir_all`
- Prevents disk leaks when download completes or errors
- Logs warning if cleanup fails, debug message on success

---

## `target/` — Build Output (ignored by Git)

Generated by `cargo build` / `cargo run`:

- **`target/debug/`** — debug build artifacts
- **`target/release/`** — release build binary
- Contains compiled `.rlib`, `.d` files, fingerprints, etc. — safe to delete and regenerate with `cargo build`

---

## Summary Diagram

```
tg-dl-bot/
├── Cargo.toml, Cargo.lock    # Dependencies
├── .env, .env.example        # Config
├── Dockerfile                # Container build
├── README.md, PLAN.md        # Docs
└── src/
    ├── main.rs               # Entry → config → bot::run
    ├── config.rs             # Env → Config
    ├── error.rs              # AppError types
    ├── bot/                  # Telegram handlers
    │   ├── mod.rs            # Dispatcher setup
    │   ├── commands.rs       # /start, /help
    │   └── handlers.rs       # Link handling, rate limit, download flow
    ├── downloader/           # Media acquisition
    │   ├── types.rs          # MediaItem, MediaResult
    │   └── ytdlp.rs          # yt-dlp subprocess
    ├── platform/             # URL classification
    │   └── matcher.rs        # URL → Platform
    ├── sender/               # Telegram upload
    │   └── upload.rs         # Size-aware send
    └── util/                 # Helpers
        ├── url.rs            # Extract URLs from text
        └── cleanup.rs        # TempDirGuard
```

---

## Data Flow (High Level)

1. **User sends message** → Telegram API (long polling)
2. **Dispatcher** → Commands go to `commands.rs`; text goes to `handlers.rs`
3. **handlers** → `extract_supported_urls()` (util) → `detect_platform()` (platform)
4. **handlers** → Rate limit check → Spawn task → `YtDlpDownloader.download()` (downloader)
5. **downloader** → yt-dlp subprocess → `MediaResult`
6. **handlers** → `MediaSender.send_media()` (sender) for each item
7. **TempDirGuard** drops → temp dir removed
