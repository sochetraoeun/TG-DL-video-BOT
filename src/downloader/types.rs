use std::path::{Path, PathBuf};

use crate::platform::matcher::Platform;

/// Shared helper to convert a file path to MediaItem (used by ytdlp and fallbacks).
pub async fn file_to_media_item(path: &Path) -> Option<MediaItem> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    if metadata.len() == 0 {
        return None;
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let media_type = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" => MediaType::Photo,
        _ => MediaType::Video,
    };

    Some(MediaItem {
        path: path.to_path_buf(),
        media_type,
        size_bytes: metadata.len(),
    })
}

#[derive(Debug, Clone)]
pub enum MediaType {
    Video,
    Photo,
}

#[derive(Debug, Clone)]
pub struct MediaItem {
    pub path: PathBuf,
    pub media_type: MediaType,
    pub size_bytes: u64,
}

#[derive(Debug)]
pub struct MediaResult {
    pub items: Vec<MediaItem>,
    pub title: Option<String>,
    pub platform: Platform,
    pub source_url: String,
}

impl MediaResult {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
