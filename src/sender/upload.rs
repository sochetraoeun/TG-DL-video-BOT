use teloxide::prelude::*;
use teloxide::types::InputFile;

use crate::config::Config;
use crate::downloader::types::{MediaItem, MediaType};

const VIDEO_SIZE_LIMIT: u64 = 50 * 1024 * 1024; // 50 MB
const PHOTO_SIZE_LIMIT: u64 = 10 * 1024 * 1024; // 10 MB

pub struct MediaSender {
    _config: Config,
}

impl MediaSender {
    pub fn new(config: &Config) -> Self {
        Self {
            _config: config.clone(),
        }
    }

    pub async fn send_media(
        &self,
        bot: &Bot,
        chat_id: ChatId,
        item: &MediaItem,
        caption: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input_file = InputFile::file(&item.path);

        match item.media_type {
            MediaType::Photo => {
                if item.size_bytes > PHOTO_SIZE_LIMIT {
                    self.send_as_document(bot, chat_id, &input_file, caption)
                        .await?;
                } else {
                    let mut req = bot.send_photo(chat_id, input_file);
                    if let Some(cap) = caption {
                        req = req.caption(cap);
                    }
                    req.await?;
                }
            }
            MediaType::Video => {
                if item.size_bytes > VIDEO_SIZE_LIMIT {
                    self.send_as_document(bot, chat_id, &input_file, caption)
                        .await?;
                } else {
                    let mut req = bot.send_video(chat_id, input_file);
                    if let Some(cap) = caption {
                        req = req.caption(cap);
                    }
                    req.await?;
                }
            }
        }

        Ok(())
    }

    async fn send_as_document(
        &self,
        bot: &Bot,
        chat_id: ChatId,
        input_file: &InputFile,
        caption: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut req = bot.send_document(chat_id, input_file.clone());
        if let Some(cap) = caption {
            req = req.caption(cap);
        }
        req.await?;
        Ok(())
    }
}
