use anyhow::Result;
use env_logger::Env;
use std::error::Error;
use std::sync::Arc;
use telegram_bot::opentmeteo::OpenMeteo;
use telegram_bot::server::BotServer;
use telegram_bot::telegram_ops::Bot;
use telegram_bot::telegram_ops::TelegramBot;
use tokio::process::Command;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

type MyBot = TelegramBot<OpenMeteo>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let bot = Arc::new(MyBot::new(OpenMeteo::new(
        "api_key".to_string(),
        "Lehnitz".to_string(),
    )));
    let server = BotServer::new("0.0.0.0", 4443, bot);
    let current_ip = server.bot.get_ip().await?;
    let webhook_ip = server.bot.get_webhook_ip().await?;
    let token = server.bot.get_token().to_string();

    let signal_handler = tokio::spawn(async {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to create signal handler terminate");
        sigterm.recv().await;
        println!("signal received. Shutting down...");
        std::process::exit(0);
    });

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    tokio::spawn(async move {
        loop {
            if !webhook_ip.is_empty() && webhook_ip != current_ip {
                println!(
                    "IP has changed(old = {}, new = {}), calling restart.sh ...",
                    current_ip, webhook_ip
                );
                let path = "/home/mohamed/personal/homebot/";
                let output = Command::new(format!("{}/restart.sh", path))
                    .arg(&current_ip)
                    .arg(&token)
                    .output()
                    .await
                    .expect("Problem with executing the command");
                println!("output is = {:#?}", String::from_utf8(output.stdout));
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        }
    });
    println!("Started the server ...");

    select! {
        _ = signal_handler => {},
        _ = server.start() => {}
    }
    Ok(())
}
