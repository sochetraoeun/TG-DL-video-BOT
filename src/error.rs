use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("download failed for {url}: {reason}")]
    Download { url: String, reason: String },

    #[error("yt-dlp exited with code {code}: {stderr}")]
    YtDlpProcess { code: i32, stderr: String },

    #[error("yt-dlp not found — is it installed and on PATH?")]
    YtDlpNotFound,

    #[error("failed to parse yt-dlp metadata: {0}")]
    MetadataParse(String),

    #[error("unsupported platform for URL: {0}")]
    UnsupportedPlatform(String),

    #[error("file too large: {path} is {size_mb:.1} MB (limit: {limit_mb} MB)")]
    FileTooLarge {
        path: PathBuf,
        size_mb: f64,
        limit_mb: u64,
    },

    #[error("telegram API error: {0}")]
    Telegram(String),

    #[error("rate limited — please wait {seconds} seconds")]
    RateLimited { seconds: u64 },

    #[error("no downloadable media found at {0}")]
    NoMedia(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
