use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "show welcome message")]
    Start,
    #[command(description = "show usage information")]
    Help,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = match cmd {
        Command::Start => {
            "Welcome to the Media Download Bot!\n\n\
             Send me a link from YouTube, TikTok, Instagram, or Facebook \
             and I'll download the video or photos for you.\n\n\
             Works in private chat and groups. In groups, mention me with a link: @botname <link>\n\n\
             Type /help for more info."
        }
        Command::Help => {
            "Supported platforms:\n\
             • YouTube (videos, shorts)\n\
             • TikTok (videos)\n\
             • Instagram (reels, posts, stories)\n\
             • Facebook (videos, reels)\n\n\
             Just paste a link and I'll handle the rest.\n\n\
             Groups: Add me to a group and mention me with a link, e.g. @botname https://...\n\
             Or disable privacy mode in @BotFather so I see all messages.\n\n\
             Note: Large files (>50 MB) will be sent as documents.\n\
             Instagram carousel photos may require cookies for full access."
        }
    };

    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}
