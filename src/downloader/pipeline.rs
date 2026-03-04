//! Download pipeline with fallback chain.
//! When yt-dlp fails for YouTube, tries rusty_ytdl then rusty_dl.

use std::path::Path;

use crate::config::Config;
use crate::error::AppError;
use crate::platform::matcher::Platform;

use super::gallery_dl_fallback;
use super::rusty_dl_fallback;
use super::rusty_ytdl_fallback;
use super::types::MediaResult;
use super::ytdlp::YtDlpDownloader;

pub struct DownloadPipeline {
    ytdlp: YtDlpDownloader,
}

impl DownloadPipeline {
    pub fn new(config: &Config) -> Self {
        Self {
            ytdlp: YtDlpDownloader::new(config),
        }
    }

    /// Primary: yt-dlp (all platforms).
    pub async fn try_ytdlp(
        &self,
        url: &str,
        platform: Platform,
        output_dir: &Path,
    ) -> Result<MediaResult, AppError> {
        self.ytdlp.download(url, platform, output_dir).await
    }

    /// Fallback 1: rusty_ytdl (YouTube only).
    pub async fn try_rusty_ytdl(&self, url: &str, output_dir: &Path) -> Result<MediaResult, AppError> {
        rusty_ytdl_fallback::download_youtube(url, output_dir).await
    }

    /// Fallback 2: rusty_dl (YouTube only).
    pub async fn try_rusty_dl(&self, url: &str, output_dir: &Path) -> Result<MediaResult, AppError> {
        rusty_dl_fallback::download_youtube(url, output_dir).await
    }

    /// Fallback for TikTok, Instagram, Facebook: gallery-dl.
    pub async fn try_gallery_dl(
        &self,
        url: &str,
        platform: Platform,
        output_dir: &Path,
    ) -> Result<MediaResult, AppError> {
        gallery_dl_fallback::download(url, platform, output_dir).await
    }
}
