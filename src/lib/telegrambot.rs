use std::path::PathBuf;

use super::types::{BotConfig, Message, Response, WeatherProvider, Webhook};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use rand::Rng;
use reqwest::{header::CONTENT_TYPE, multipart, Body};
use serde_json::{json, Value};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[async_trait]
pub trait Bot: Send + Sync + 'static {
    async fn handle_message(&self, msg: Message) -> Result<()>;
    async fn get_webhook_ip(&self) -> Result<String>;
    fn get_webhook_ips(&self) -> Result<Vec<&'static str>>;
}

#[derive(Clone)]
pub struct TelegramBot<T: WeatherProvider> {
    client: reqwest::Client,
    weather: T,
    config: BotConfig,
}

impl<T: WeatherProvider> TelegramBot<T> {
    pub fn new(weather: T, config: BotConfig) -> Self {
        TelegramBot {
            client: reqwest::Client::new(),
            config: config,
            weather: weather,
        }
    }

    pub fn get_token(&self) -> &str {
        &self.config.token
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
            .context("Could not send the reply")?;
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

    pub async fn update_webhook_cert(&self, cert: PathBuf, ip: &str) -> Result<()> {
        // get the pubkey file
        let certificate = File::open(cert).await?;
        let stream = FramedRead::new(certificate, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);
        let cert_form = multipart::Part::stream(file_body);

        let url = format!(
            "https://api.telegram.org/bot{}/setWebhook",
            self.config.token
        );

        let form = multipart::Form::new()
            .text("url", format!("https://{}", ip))
            .part("certificate", cert_form);

        self.client
            .post(url)
            .multipart(form)
            .send()
            .await
            .context("Could not set the webhook")?;
        Ok(())
    }
}

#[async_trait]
impl<T: WeatherProvider + 'static> Bot for TelegramBot<T> {
    async fn handle_message(&self, msg: Message) -> Result<()> {
        let answer: String;
        let id = msg.chat.id;
        let mut command = msg.text.split_whitespace();
        answer = match command.next() {
            Some("/ip") => {
                if let Ok(ip) = self.get_ip().await {
                    ip
                } else {
                    "Problem getting the ip, try again".into()
                }
            }
            Some("/temp") => {
                let mut city = self.weather.get_favourite_city();
                if let Some(arg) = command.next() {
                    city = arg.to_string();
                }
                if let Some(temp) = self.weather.get_temperature(city).await {
                    temp.to_string()
                } else {
                    "Error getting the temp".into()
                }
            }
            Some("/dice") => rand::thread_rng().gen_range(1..=6).to_string(),
            Some("hello") => "hello back :)".into(),
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

    fn get_webhook_ips(&self) -> Result<Vec<&'static str>> {
        // allow the telegram servers IP address
        // According to https://core.telegram.org/bots/webhooks
        // the allowed IP addresses would be 149.154.160.0/20 and 91.108.4.0/22
        Ok(vec![
            "91.108.4.*",
            "91.108.5.*",
            "91.108.6.*",
            "91.108.7.*",
            "149.154.16?.*",
            "149.154.17?.*",
        ])
    }
}

#[cfg(test)]
mod test {
    fn test_new() {}
}
