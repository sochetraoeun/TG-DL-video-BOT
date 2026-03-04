use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters};
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::downloader::pipeline::DownloadPipeline;
use crate::sender::upload::MediaSender;
use crate::util::cleanup::TempDirGuard;
use crate::util::url::extract_supported_urls;

pub struct State {
    rate_limits: DashMap<UserId, Instant>,
    rate_limit_duration: Duration,
    semaphore: Semaphore,
}

impl State {
    pub fn new(config: &Config) -> Self {
        Self {
            rate_limits: DashMap::new(),
            rate_limit_duration: Duration::from_secs(config.rate_limit_seconds),
            semaphore: Semaphore::new(config.max_concurrent_downloads),
        }
    }

    fn check_rate_limit(&self, user_id: UserId) -> Result<(), u64> {
        if let Some(last) = self.rate_limits.get(&user_id) {
            let elapsed = last.elapsed();
            if elapsed < self.rate_limit_duration {
                let remaining = (self.rate_limit_duration - elapsed).as_secs() + 1;
                return Err(remaining);
            }
        }
        self.rate_limits.insert(user_id, Instant::now());
        Ok(())
    }
}

pub async fn handle_message(
    bot: Bot,
    msg: Message,
    downloader: Arc<DownloadPipeline>,
    media_sender: Arc<MediaSender>,
    state: Arc<State>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match msg.text() {
        Some(t) => t,
        None => return Ok(()),
    };

    let urls = extract_supported_urls(text);
    if urls.is_empty() {
        return Ok(());
    }

    let user_id = match msg.from.as_ref() {
        Some(user) => user.id,
        None => return Ok(()),
    };

    if let Err(remaining_secs) = state.check_rate_limit(user_id) {
        bot.send_message(
            msg.chat.id,
            format!("Please wait {remaining_secs}s before the next download."),
        )
        .await?;
        return Ok(());
    }

    for extracted in &urls {
        let chat_id = msg.chat.id;
        let url = extracted.url.clone();
        let platform = extracted.platform;

        let status_msg = bot
            .send_message(
                chat_id,
                format!(
                    "Downloading from {}...",
                    platform.display_name()
                ),
            )
            .parse_mode(ParseMode::Html)
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;

        let bot_clone = bot.clone();
        let downloader = downloader.clone();
        let media_sender = media_sender.clone();
        let state_ref = state.clone();

        tokio::spawn(async move {
            let _permit = match state_ref.semaphore.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };

            let result =
                process_download(&bot_clone, chat_id, &url, platform, &downloader, &media_sender, &status_msg)
                    .await;

            if let Err(e) = result {
                tracing::error!(%url, error = %e, "download pipeline failed");
                let _ = bot_clone
                    .edit_message_text(chat_id, status_msg.id, format!("Failed: {e}"))
                    .await;
            }
        });
    }

    Ok(())
}

async fn process_download(
    bot: &Bot,
    chat_id: ChatId,
    url: &str,
    platform: crate::platform::matcher::Platform,
    downloader: &DownloadPipeline,
    media_sender: &MediaSender,
    status_msg: &Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tmp_dir = tempfile::tempdir()?;
    let _guard = TempDirGuard::new(tmp_dir.path().to_path_buf());
    let output_dir = tmp_dir.path();

    use crate::platform::matcher::Platform;

    let result = match downloader.try_ytdlp(url, platform, output_dir).await {
        Ok(r) => r,
        Err(ytdlp_err) => {
            match platform {
                Platform::YouTube => {
                    tracing::warn!(%url, error = %ytdlp_err, "yt-dlp failed, trying rusty_ytdl");

                    let _ = bot
                        .edit_message_text(
                            chat_id,
                            status_msg.id,
                            "Downloading... (trying alternative 1/2)",
                        )
                        .await;

                    match downloader.try_rusty_ytdl(url, output_dir).await {
                        Ok(r) => r,
                        Err(rusty_ytdl_err) => {
                            tracing::warn!(%url, error = %rusty_ytdl_err, "rusty_ytdl failed, trying rusty_dl");

                            let _ = bot
                                .edit_message_text(
                                    chat_id,
                                    status_msg.id,
                                    "Downloading... (trying alternative 2/2)",
                                )
                                .await;

                            match downloader.try_rusty_dl(url, output_dir).await {
                                Ok(r) => r,
                                Err(e) => {
                                    tracing::error!(%url, "all download methods failed");
                                    return Err(e.into());
                                }
                            }
                        }
                    }
                }
                Platform::TikTok | Platform::Instagram | Platform::Facebook => {
                    tracing::warn!(%url, error = %ytdlp_err, "yt-dlp failed, trying gallery-dl");

                    let _ = bot
                        .edit_message_text(
                            chat_id,
                            status_msg.id,
                            "Downloading... (trying alternative)",
                        )
                        .await;

                    match downloader.try_gallery_dl(url, platform, output_dir).await {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::error!(%url, "yt-dlp and gallery-dl failed");
                            return Err(e.into());
                        }
                    }
                }
            }
        }
    };

    bot.edit_message_text(
        chat_id,
        status_msg.id,
        format!("Uploading {} file(s)...", result.items.len()),
    )
    .await?;

    let caption = result
        .title
        .as_deref()
        .map(|t| truncate(t, 200).to_string());

    for (i, item) in result.items.iter().enumerate() {
        let cap = if i == 0 { caption.as_deref() } else { None };
        media_sender.send_media(bot, chat_id, item, cap).await?;
    }

    let _ = bot.delete_message(chat_id, status_msg.id).await;

    tracing::info!(
        %url,
        platform = platform.display_name(),
        items = result.items.len(),
        "download complete"
    );

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> &str {
    match s.char_indices().nth(max_len) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

pub struct LogErrorHandler;

impl teloxide::error_handlers::ErrorHandler<Box<dyn std::error::Error + Send + Sync>>
    for LogErrorHandler
{
    fn handle_error(
        self: Arc<Self>,
        error: Box<dyn std::error::Error + Send + Sync>,
    ) -> futures::future::BoxFuture<'static, ()> {
        tracing::error!(error = %error, "unhandled dispatcher error");
        Box::pin(async {})
    }
}
