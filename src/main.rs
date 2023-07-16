use anyhow::Result;
use telegram_bot::server::BotServer;
use telegram_bot::bot::Bot;
use std::{error::Error, sync::Arc};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let homebot = Bot::new();
    let current_ip = if let Ok(ip) = homebot.get_ip().await {ip} else {"0.0.0.0".to_string()};
    let homebot_ref = Arc::new(homebot);
    let homebot_ref_clone = homebot_ref.clone();
    let mut first_time = true;
    tokio::spawn(async move {
        loop {
            let result = homebot_ref_clone.get_ip().await;
            if let Ok(ip) = result {
                // TODO: Check against the ip of the webhook
                if (!ip.is_empty() && ip != current_ip) || (first_time == true) {
                    println!(
                        "IP has changed(old = {}, new = {}), calling restart.sh ...",
                        current_ip, ip
                    );
                    let path = "/home/mohamed/personal/homebot/";
                    let output = Command::new(format!("{}/restart.sh", path))
                        .arg(ip)
                        .arg(&homebot_ref_clone.get_token())
                        .output()
                        .await
                        .expect("Problem with executing the command");
                    println!("output is = {:#?}", String::from_utf8(output.stdout));
                    first_time = false;
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                }
            }
        }
    });
    println!("Started the server ...");
    let server: BotServer = BotServer::new("0.0.0.0", 4443, homebot_ref);
    server.start().await?;
    Ok(())
}

