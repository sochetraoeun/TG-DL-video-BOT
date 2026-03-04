mod bot;
mod config;
mod downloader;
mod error;
mod platform;
mod sender;
mod util;

use config::Config;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tg_dl_bot=info".into()),
        )
        .init();

    let config = Config::from_env();
    tracing::info!(
        max_concurrent = config.max_concurrent_downloads,
        rate_limit_secs = config.rate_limit_seconds,
        "starting tg-dl-bot"
    );

    bot::run(config).await;
}
