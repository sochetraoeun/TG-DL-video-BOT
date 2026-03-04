use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub max_concurrent_downloads: usize,
    pub rate_limit_seconds: u64,
    pub cookies_path: Option<PathBuf>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            max_concurrent_downloads: env::var("MAX_CONCURRENT_DOWNLOADS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            rate_limit_seconds: env::var("RATE_LIMIT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            cookies_path: env::var("COOKIES_PATH").ok().map(PathBuf::from),
        }
    }
}
