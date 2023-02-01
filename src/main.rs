mod server;
mod types;
use anyhow::{bail, Context, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::{json, Value};
use server::BotServer;
use std::error::Error;
use std::fs;
use toml;
use types::{Message, Response, Webhook};

/// Layout of config.toml should be like:
/// [bot]
/// name = "superbot"
/// token = "11111111112222222222333333333"
#[derive(Deserialize, Debug)]

struct Config {
    bot: BotConfig,
}
#[derive(Deserialize, Debug)]
struct BotConfig {
    name: String,
    token: String,
}

pub struct Bot {
    client: reqwest::Client,
    current_ip: Option<String>,
    server: BotServer,
    config: BotConfig,
}

impl Bot {
    fn new(ip: &'static str, port: u32) -> Self {
        let conf = Bot::get_config().unwrap();
        Bot {
            current_ip: None,
            client: reqwest::Client::new(),
            server: BotServer::new(ip, port),
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
        Ok(ip["origin"].to_string())
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

    // fn disable_webhook() -> Result<()> {}
    async fn get_webhook_ip(&self) -> Result<String> {
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
    let homebot = Bot::new("0.0.0.0", 4443);

    let ip = homebot.get_ip().await?;
    println!("{:#?}", ip);
    let hook = homebot.get_webhook_ip().await?;
    println!("{:#?}", hook);
    homebot.server.start().await?;
    Ok(())
}
