mod types;
use anyhow::{bail, Context, Result};
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};
use std::error::Error;
use tokio::{
    select,
    time::{sleep, Duration},
};
use types::{Message, Response};

pub struct Bot {
    cycle: Duration,
    last_update: u64,
    updates_limit: u32,
    client: reqwest::Client,
    current_ip: Option<String>,
}

impl Bot {
    /// TODO: setup a web server, that listens for post requests,
    /// adds our ip address, as a webhook with a defined secret
    /// changes the webhook everytime our ip address changes.
    const TOKEN: &str = "";
    const NAME: &str = "";

    async fn new(cycle: u64, limit: u32) -> Self {
        let id: u64 = Bot::get_last_id().await.unwrap_or(0);
        Bot {
            cycle: Duration::from_millis(cycle),
            last_update: id,
            updates_limit: limit,
            current_ip: None,
            client: reqwest::Client::new(),
        }
    }

    async fn get_last_id() -> Result<u64> {
        let url = format!("https://api.telegram.org/bot{}/getUpdates", Bot::TOKEN);
        let resp: Response = reqwest::get(url).await?.text().await?.into();
        if resp.ok {
            if let Some(update) = resp.result.last() {
                println!("last update id {:#?}", update.update_id);
                return Ok(update.update_id);
            }
        }
        bail!("Problem with getting last id")
    }

    async fn reply(&self, id: u64, msg: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", Bot::TOKEN);
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

    async fn get_last_message(&mut self) -> Result<Option<Message>> {
        let url = format!("https://api.telegram.org/bot{}/getUpdates", Bot::TOKEN);
        let resp: Response = self
            .client
            .get(url)
            .header(CONTENT_TYPE, "application/json")
            .body(json!({"offset": self.last_update, "limit": self.updates_limit}).to_string())
            .send()
            .await?
            .text()
            .await?
            .into();

        if resp.ok {
            if let Some(update) = resp.result.last() {
                if update.update_id > self.last_update {
                    self.last_update = update.update_id + 1;
                }
                self.last_update += 1;
                println!("Got {:#?}", update.message);
                return Ok(Some(update.message.clone()));
            } else {
                return Ok(None);
            }
        }
        bail!("Error while getting the update")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut homebot = Bot::new(500, 1).await;
    let ip = homebot.get_ip().await?;
    println!("{:#?}", ip);
    loop {
        sleep(homebot.cycle).await;
        select! {
            Ok(Some(msg)) = homebot.get_last_message() => {
                let answer: &str;
                let id = msg.chat.id;
                match msg.text.as_str() {
                    "/ip" => answer = &ip,
                    "hello" => answer = "hello back :)",
                    _ => answer = "did not understand!",
                }
                homebot.reply(id, answer).await?;
            },

            else => continue,
        };
    }
}
