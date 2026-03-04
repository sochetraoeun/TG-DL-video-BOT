//! Fallback downloader using gallery-dl (TikTok, Instagram, Facebook).
//! Used when yt-dlp fails for these platforms.

use std::path::{Path, PathBuf};

use tokio::process::Command;

use crate::error::AppError;
use crate::platform::matcher::Platform;

use super::types::{file_to_media_item, MediaItem, MediaResult};

pub async fn download(
    url: &str,
    platform: Platform,
    output_dir: &Path,
) -> Result<MediaResult, AppError> {
    let output = Command::new("gallery-dl")
        .arg("-D")
        .arg(output_dir)
        .arg("--no-mtime")
        .arg(url)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::Download {
                    url: url.to_string(),
                    reason: "gallery-dl not found — install with: pip install gallery-dl".to_string(),
                }
            } else {
                AppError::Download {
                    url: url.to_string(),
                    reason: format!("gallery-dl: {e}"),
                }
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Download {
            url: url.to_string(),
            reason: format!("gallery-dl: {}", stderr.lines().next().unwrap_or(&stderr)),
        });
    }

    let items = scan_for_media_recursive(output_dir).await;
    if items.is_empty() {
        return Err(AppError::NoMedia(url.to_string()));
    }

    Ok(MediaResult {
        items,
        title: None,
        platform,
        source_url: url.to_string(),
    })
}

async fn scan_for_media_recursive(dir: &Path) -> Vec<MediaItem> {
    let mut items = Vec::new();
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(mut read_dir) = tokio::fs::read_dir(&current).await {
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file()
                    && path
                        .extension()
                        .map_or(false, |ext| !ext.to_string_lossy().to_lowercase().ends_with("json"))
                {
                    if let Some(item) = file_to_media_item(&path).await {
                        items.push(item);
                    }
                }
            }
        }
    }
    items
}
