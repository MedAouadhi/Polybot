use anyhow::Ok;
use anyhow::Result;
use env_logger::Env;
use std::env;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use telegram_bot::openmeteo::OpenMeteo;
use telegram_bot::server::BotServer;
use telegram_bot::telegrambot::Bot;
use telegram_bot::telegrambot::TelegramBot;
use telegram_bot::Config;
use tokio::fs;
use tokio::process::Command;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

type MyBot = TelegramBot<OpenMeteo>;

async fn get_config() -> Result<Config> {
    let mut config_file = PathBuf::from(env::current_dir().unwrap());
    config_file.push("config.toml");
    let toml_str = fs::read_to_string(config_file).await?;
    let map: Config = toml::from_str(&toml_str)?;
    println!("{:#?}", map);
    Ok(map)
}

async fn generate_certificate(name: &'static str, ip: &str) -> Result<PathBuf> {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = get_config().await?;
    let bot = Arc::new(MyBot::new(
        OpenMeteo::new(None, "Lehnitz".to_string()),
        conf.bot,
    ));
    let server = BotServer::new(conf.server, bot);
    let current_ip = server.bot.get_ip().await?;
    let webhook_ip = server.bot.get_webhook_ip().await?;
    let token = server.bot.get_token().to_string();

    let signal_handler = tokio::spawn(async {
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to create signal handler terminate");
        sigterm.recv().await;
        println!("signal received. Shutting down...");
        std::process::exit(0);
    });

    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    tokio::spawn(async move {
        loop {
            if !webhook_ip.is_empty() && webhook_ip != current_ip {
                println!(
                    "IP has changed(old = {}, new = {}), calling restart.sh ...",
                    current_ip, webhook_ip
                );
                let mut restart_script = PathBuf::from(env::current_dir().unwrap());
                restart_script.push("restart.sh");
                let output = Command::new(restart_script)
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
