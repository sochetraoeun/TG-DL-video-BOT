//! Fallback downloader using rusty_dl (YouTube only).
//! Used when yt-dlp and rusty_ytdl both fail for YouTube URLs.

use std::path::Path;

use rusty_dl::prelude::{Downloader, YoutubeDownloader};

use crate::error::AppError;
use crate::platform::matcher::Platform;

use super::types::{file_to_media_item, MediaResult};

pub async fn download_youtube(url: &str, output_dir: &Path) -> Result<MediaResult, AppError> {
    let downloader = YoutubeDownloader::new(url).map_err(|e| AppError::Download {
        url: url.to_string(),
        reason: format!("rusty_dl: {e}"),
    })?;

    downloader.download_to(output_dir).await.map_err(|e| AppError::Download {
        url: url.to_string(),
        reason: format!("rusty_dl: {e}"),
    })?;

    let items = scan_for_media(output_dir).await;
    if items.is_empty() {
        return Err(AppError::NoMedia(url.to_string()));
    }

    Ok(MediaResult {
        items,
        title: None,
        platform: Platform::YouTube,
        source_url: url.to_string(),
    })
}

async fn scan_for_media(dir: &Path) -> Vec<super::types::MediaItem> {
    let mut items = Vec::new();
    let mut read_dir = match tokio::fs::read_dir(dir).await {
        Ok(rd) => rd,
        Err(_) => return items,
    };

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| !ext.to_string_lossy().to_lowercase().ends_with("json"))
        {
            if let Some(item) = file_to_media_item(&path).await {
                items.push(item);
            }
        }
    }

    items
}
