mod server;
mod types;
use anyhow::{bail, Context, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::{json, Value};
use server::BotServer;
use std::fs;
use std::{error::Error, sync::Arc};
use tokio::process::Command;
use toml;
use types::{Message, Response, Webhook};

#[derive(Deserialize, Debug)]
struct Config {
    bot: BotConfig,
}

#[derive(Deserialize, Debug, Clone)]
struct BotConfig {
    name: String,
    token: String,
}

#[derive(Clone)]
pub struct Bot {
    client: reqwest::Client,
    current_ip: Option<String>,
    config: BotConfig,
}

impl Bot {
    fn new() -> Self {
        let conf = Bot::get_config().unwrap();
        Bot {
            current_ip: None,
            client: reqwest::Client::new(),
            config: conf.bot,
        }
    }

    fn get_config() -> Result<Config> {
        let toml_str = fs::read_to_string("config.toml")?;
        let map: Config = toml::from_str(&toml_str)?;
        println!("{:#?}", map);
        Ok(map)
    }

    async fn reply(&self, id: u64, msg: &str) -> Result<()> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.config.token
        );
        self.client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .body(json!({"chat_id": id, "text": msg}).to_string())
            .send()
            .await
            .context("Could send the reply")?;
        Ok(())
    }

    async fn get_ip(&self) -> Result<String> {
        let resp: String = self
            .client
            .get("https://httpbin.org/ip")
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .text()
            .await?;
        let ip: Value = serde_json::from_str(&resp).context("Failed to parse the json output")?;
        Ok(ip["origin"].to_string().replace('"', ""))
    }

    async fn handle_message(&self, msg: Message) -> Result<()> {
        let answer: String;
        let id = msg.chat.id;
        answer = match msg.text.as_str() {
            "/ip" => self.get_ip().await?,
            "hello" => "hello back :)".into(),
            _ => "did not understand!".into(),
        };
        self.reply(id, &answer).await?;
        Ok(())
    }

    async fn get_webhook_ip(&self) -> Result<String> {
        //gets the web hook info, we use to know if the ip address set in the certificate
        //is correct or not.
        let url = format!(
            "https://api.telegram.org/bot{}/getWebhookInfo",
            self.config.token
        );
        let resp: Response<Webhook> = self.client.get(url).send().await?.text().await?.into();

        if resp.ok {
            return Ok(resp.result.ip_address.clone());
        } else {
            bail!("Could not get correct webhook");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let homebot = Bot::new();
    let current_ip = homebot.get_ip().await?;
    let homebot_ref = Arc::new(homebot);
    let homebot_ref_clone = homebot_ref.clone();
    tokio::spawn(async move {
        loop {
            let result = homebot_ref_clone.get_ip().await;
            if let Ok(ip) = result {
                // TODO: Check against the ip of the webhook
                if !ip.is_empty() && ip != current_ip {
                    println!(
                        "IP has changed(old = {}, new = {}), calling restart.sh ...",
                        current_ip, ip
                    );
                    let path = std::env::current_dir()
                        .unwrap()
                        .into_os_string()
                        .into_string()
                        .unwrap();
                    let output = Command::new(format!("{}/restart.sh", path))
                        .arg(ip)
                        .arg(&homebot_ref_clone.config.token)
                        .output()
                        .await
                        .expect("Problem with executing the command");
                    println!("output is = {:#?}", String::from_utf8(output.stdout));
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
