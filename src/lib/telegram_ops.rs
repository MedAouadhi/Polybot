use anyhow::{bail, Context, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::{json, Value};
use super::server::BotServer;
use std::fs;
use toml;
use super::types::{Message, Response, Webhook};

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
    pub fn new() -> Self {
        let conf = Bot::get_config().unwrap();
        Bot {
            current_ip: None,
            client: reqwest::Client::new(),
            config: conf.bot,
        }
    }

    pub fn get_token(&self) -> &str {
       &self.config.token
    }

    fn get_config() -> Result<Config> {
        let path = std::env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        
        let file = format!("/home/mohamed/personal/homebot/config.toml"); 
        println!("{:?}", file);
        let toml_str = fs::read_to_string(file)?;
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

    pub async fn get_ip(&self) -> Result<String> {
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

    pub async fn handle_message(&self, msg: Message) -> Result<()> {
        let answer: String;
        let id = msg.chat.id;
        answer = match msg.text.as_str() {
            "/ip" => if let Ok(ip) = self.get_ip().await {ip} else {"Problem getting the ip, try again".into()},
            "hello" => "hello back :)".into(),
            _ => "did not understand!".into(),
        };
        self.reply(id, &answer).await?;
        Ok(())
    }

    pub async fn get_webhook_ip(&self) -> Result<String> {
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
    //TODO: implement get_web_hook
}
