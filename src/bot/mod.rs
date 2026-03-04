pub mod commands;
pub mod handlers;

use std::sync::Arc;

use teloxide::dispatching::UpdateFilterExt;
use teloxide::dptree;
use teloxide::prelude::*;

use crate::config::Config;
use crate::downloader::pipeline::DownloadPipeline;
use crate::sender::upload::MediaSender;

use self::commands::Command;
use self::handlers::State;

pub async fn run(config: Config) {
    let bot = Bot::from_env();

    let downloader = Arc::new(DownloadPipeline::new(&config));
    let media_sender = Arc::new(MediaSender::new(&config));
    let state = Arc::new(State::new(&config));

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(commands::handle_command),
        )
        .branch(dptree::entry().endpoint(handlers::handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![downloader, media_sender, state])
        .default_handler(|_| async {})
        .error_handler(Arc::new(handlers::LogErrorHandler))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
