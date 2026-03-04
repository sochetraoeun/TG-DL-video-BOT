use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio::process::Command;

use crate::config::Config;
use crate::error::AppError;
use crate::platform::matcher::Platform;

use super::types::{file_to_media_item, MediaItem, MediaResult};

pub struct YtDlpDownloader {
    cookies_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct YtDlpMetadata {
    title: Option<String>,
    #[serde(rename = "_type")]
    entry_type: Option<String>,
    entries: Option<Vec<YtDlpMetadata>>,
    ext: Option<String>,
    #[serde(default)]
    requested_downloads: Vec<RequestedDownload>,
}

#[derive(Debug, Deserialize)]
struct RequestedDownload {
    filepath: Option<String>,
}

impl YtDlpDownloader {
    pub fn new(config: &Config) -> Self {
        Self {
            cookies_path: config.cookies_path.clone(),
        }
    }

    pub async fn download(
        &self,
        url: &str,
        platform: Platform,
        output_dir: &Path,
    ) -> Result<MediaResult, AppError> {
        self.ensure_ytdlp_available().await?;

        let output_template = output_dir.join("%(title).80s_%(id)s.%(ext)s");

        let mut cmd = Command::new("yt-dlp");
        cmd.arg("--no-playlist")
            .arg("--write-info-json")
            .arg("--print-json")
            .arg("-o")
            .arg(&output_template);

        self.apply_platform_args(&mut cmd, platform);

        if let Some(ref cookies) = self.cookies_path {
            cmd.arg("--cookies").arg(cookies);
        }

        cmd.arg(url);

        tracing::info!(%url, platform = platform.display_name(), "starting yt-dlp download");

        let output = cmd.output().await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::YtDlpNotFound
            } else {
                AppError::Io(e)
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().unwrap_or(-1);
            tracing::error!(%url, %code, %stderr, "yt-dlp failed");
            return Err(AppError::YtDlpProcess { code, stderr });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_output(&stdout, platform, url, output_dir)
            .await
    }

    fn apply_platform_args(&self, cmd: &mut Command, platform: Platform) {
        match platform {
            Platform::YouTube => {
                // Prefer mp4 up to 1080p to stay under Telegram's 50 MB limit
                cmd.arg("-f")
                    .arg("bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best");
            }
            Platform::TikTok => {
                cmd.arg("-f").arg("best[ext=mp4]/best");
            }
            Platform::Instagram => {
                cmd.arg("-f").arg("best");
            }
            Platform::Facebook => {
                cmd.arg("-f")
                    .arg("best[ext=mp4]/best");
            }
        }
    }

    async fn parse_output(
        &self,
        stdout: &str,
        platform: Platform,
        source_url: &str,
        output_dir: &Path,
    ) -> Result<MediaResult, AppError> {
        let mut items = Vec::new();
        let mut title = None;

        // yt-dlp --print-json outputs one JSON object per line
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let metadata: YtDlpMetadata = serde_json::from_str(line).map_err(|e| {
                AppError::MetadataParse(format!("{e}: {}", &line[..line.len().min(200)]))
            })?;

            if title.is_none() {
                title = metadata.title.clone();
            }

            self.collect_items_from_metadata(&metadata, output_dir, &mut items)
                .await;
        }

        // Fallback: scan output_dir for any media files yt-dlp created
        if items.is_empty() {
            self.scan_output_dir(output_dir, &mut items).await;
        }

        if items.is_empty() {
            return Err(AppError::NoMedia(source_url.to_string()));
        }

        Ok(MediaResult {
            items,
            title,
            platform,
            source_url: source_url.to_string(),
        })
    }

    async fn collect_items_from_metadata(
        &self,
        metadata: &YtDlpMetadata,
        output_dir: &Path,
        items: &mut Vec<MediaItem>,
    ) {
        for dl in &metadata.requested_downloads {
            if let Some(ref filepath) = dl.filepath {
                let path = PathBuf::from(filepath);
                if let Some(item) = file_to_media_item(&path).await {
                    items.push(item);
                }
            }
        }

        if let Some(ref entries) = metadata.entries {
            for entry in entries {
                Box::pin(self.collect_items_from_metadata(entry, output_dir, items)).await;
            }
        }
    }

    async fn scan_output_dir(&self, dir: &Path, items: &mut Vec<MediaItem>) {
        let mut read_dir = match tokio::fs::read_dir(dir).await {
            Ok(rd) => rd,
            Err(_) => return,
        };

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if path.extension().map_or(false, |ext| {
                let ext = ext.to_string_lossy().to_lowercase();
                // Skip yt-dlp info JSON files
                !ext.ends_with("json")
            }) {
                if let Some(item) = file_to_media_item(&path).await {
                    items.push(item);
                }
            }
        }
    }

    async fn ensure_ytdlp_available(&self) -> Result<(), AppError> {
        Command::new("yt-dlp")
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::YtDlpNotFound
                } else {
                    AppError::Io(e)
                }
            })?;
        Ok(())
    }
}
