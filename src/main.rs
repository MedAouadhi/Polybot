use anyhow::Result;
use telegram_bot::server::BotServer;
use std::{error::Error, sync::Arc};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server: BotServer = BotServer::new("0.0.0.0", 4443);
    let current_ip = server.bot.get_ip().await?;
    let webhook_ip = server.bot.get_webhook_ip().await?;
    let token = server.bot.get_token().to_string();

    tokio::spawn(async move {
        loop {
                // TODO: Check against the ip of the webhook
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
    server.start().await?;
    Ok(())
}

