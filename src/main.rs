mod bot_commands;
use anyhow::Result;
use bot_commands::commands::MyCommands;
use polybot::polybot::Polybot;
use polybot::telegram::bot::TelegramBot;
use std::error::Error;
use std::time::Duration;
use tracing::info;

// MyCommands is the macro generated struct that holds the list of commands
// defined in bot_commands.rs and implements BotCommands trait.
type MyBot = TelegramBot<MyCommands>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = polybot::utils::get_config("config.toml").await?;
    let telegrambot =
        Polybot::<MyBot>::new(config).with_webhook_monitoring(Duration::from_secs(60));

    info!("Starting Telegram Bot ...");
    telegrambot.start_loop().await?;
    Ok(())
}
