//! Fallback downloader using rusty_ytdl (YouTube only).
//! Used when yt-dlp fails for YouTube URLs.

use std::path::Path;

use rusty_ytdl::Video;

use crate::error::AppError;
use crate::platform::matcher::Platform;

use super::types::{file_to_media_item, MediaItem, MediaResult};

pub async fn download_youtube(url: &str, output_dir: &Path) -> Result<MediaResult, AppError> {
    let video = Video::new(url).map_err(|e| AppError::Download {
        url: url.to_string(),
        reason: format!("rusty_ytdl: {e}"),
    })?;

    let video_id = video.get_video_id();
    let output_path = output_dir.join(format!("{}.mp4", video_id));

    video.download(&output_path).await.map_err(|e| AppError::Download {
        url: url.to_string(),
        reason: format!("rusty_ytdl: {e}"),
    })?;

    let item = file_to_media_item(&output_path)
        .await
        .or(scan_first_media(output_dir).await)
        .ok_or_else(|| AppError::NoMedia(url.to_string()))?;

    let title = video
        .get_basic_info()
        .await
        .ok()
        .map(|info| info.video_details.title);

    Ok(MediaResult {
        items: vec![item],
        title,
        platform: Platform::YouTube,
        source_url: url.to_string(),
    })
}

async fn scan_first_media(dir: &Path) -> Option<MediaItem> {
    let mut read_dir = tokio::fs::read_dir(dir).await.ok()?;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| !ext.to_string_lossy().to_lowercase().ends_with("json"))
        {
            if let Some(item) = file_to_media_item(&path).await {
                return Some(item);
            }
        }
    }
    None
}
