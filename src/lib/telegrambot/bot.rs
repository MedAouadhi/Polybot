use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;

use crate::telegrambot::types::{Response, Update, Webhook};
use crate::types::{
    Bot, BotCommands, BotConfig, BotMessage, BotMessages, BotUser, CommandHashMap, SharedUsers,
};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use reqwest::multipart::Part;
use reqwest::{header::CONTENT_TYPE, multipart};
use serde_json::json;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::debug;

pub struct TelegramBot<P: BotCommands> {
    client: reqwest::Client,
    config: BotConfig,
    users: SharedUsers,
    command_list: CommandHashMap,
    _commands: PhantomData<P>,
}

impl<P: BotCommands> TelegramBot<P> {
    pub fn new(config: BotConfig) -> Self {
        TelegramBot {
            client: reqwest::Client::new(),
            config,
            users: Arc::new(Mutex::new(HashMap::new())),
            command_list: P::command_list(),
            _commands: PhantomData::default(),
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

    pub async fn update_webhook_cert(&self, cert: PathBuf, ip: &str) -> Result<()> {
        // get the pubkey file
        let certificate = fs::read(&cert)
            .await
            .expect("Failed to read the certificate file");

        let url = format!(
            "https://api.telegram.org/bot{}/setWebhook",
            self.config.token
        );

        let part = Part::bytes(certificate).file_name("cert.pem");
        let form = multipart::Form::new()
            .text("url", format!("https://{}", ip))
            .part("certificate", part);

        let resp = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "multipart/form-data")
            .multipart(form)
            .send()
            .await
            .context("Could not set the webhook")?;
        debug!("[webhook set]{:#?}", resp.text().await);
        Ok(())
    }
}

#[async_trait]
impl<P: BotCommands + Default + 'static> Bot for TelegramBot<P> {
    async fn handle_message(&self, msg: String) -> Result<()> {
        let answer: String;
        let id: u64;
        let update: Update = msg.into();
        debug!("Received {:#?}", update);
        if let Some(message) = update.message {
            let msg = BotMessages::from(message);
            id = msg.get_chat_id();
            let (user_id, user_name) = msg.get_user();
            let command;
            let argument;

            let mut users = self.users.lock().await;
            if users.get(&user_id).is_none() {
                // add the user in the hashmap
                debug!(
                    "Adding the user (id = {}), (name = {}).",
                    user_id, user_name
                );
                users.insert(user_id, Mutex::new(BotUser::new()));
            };

            let text = msg.get_message();
            // Create a context so that we unlock the user right before calling the handler
            {
                let mut user = users.get_mut(&user_id).unwrap().lock().await;

                // update the user activity
                user.set_last_activity(chrono::Utc::now());

                // if we are in chat mode, interpret the message as llm ask request
                if user.is_in_chat_mode() && !text.starts_with(&P::chat_exit_command().unwrap()) {
                    command = P::llm_request_command();
                    argument = text;
                } else {
                    let mut message = text.split_whitespace();
                    command = message.next();
                    argument = message.collect::<Vec<&str>>().join(" ");
                }
                debug!("Cmd: {:?}, Arg: {:?}", command, argument);
            }
            answer = if let Some(bot_command) = self.command_list.get(command.unwrap()) {
                bot_command
                    .handle(Arc::new(Mutex::new(BotUser::new())), argument)
                    .await
            } else {
                "Did not understand!".into()
            };
        } else {
            bail!("Unsupported message format!");
        }
        // let answer = handler.await;
        // answer = match command {
        //     Some("/temp") => {
        //         let mut city = self.weather.get_favourite_city();
        //         if !argument.is_empty() {
        //             city = argument;
        //         }
        //         if let Some(temp) = self.weather.get_temperature(city).await {
        //             temp.to_string()
        //         } else {
        //             "Error getting the temp".into()
        //         }
        //     }
        //     Some("/dice") => rand::thread_rng().gen_range(1..=6).to_string(),
        //     Some("/affirm") => self.get_affirmation().await?,
        //     Some("/ask") => {
        //         if let Some(ref agent) = self.llm_agent {
        //             if !argument.is_empty() {
        //                 // TODO: Remove the unwrap
        //                 agent.request(&argument).await.unwrap()
        //             } else {
        //                 "You need to ask something".into()
        //             }
        //         } else {
        //             "Agent not configured!".into()
        //         }
        //     }
        //     Some("/chat") => {
        //         debug!("Entering llm chat mode");
        //         user.set_chat_mode(true);
        //         "Let's talk!".into()
        //     }
        //     Some("/endchat") => {
        //         debug!("Exiting llm chat mode");
        //         user.set_chat_mode(false);
        //         "See ya!".into()
        //     }
        //     Some("hello") => "hello back :)".into(),
        //     _ => "did not understand!".into(),
        // };
        self.reply(id, &answer).await?;
        Ok(())
    }

    async fn is_webhook_configured(&self, ip: &str) -> Result<bool> {
        //gets the web hook info, we use to know if the ip address set in the certificate
        //is correct or not.
        let url = format!(
            "https://api.telegram.org/bot{}/getWebhookInfo",
            self.config.token
        );
        let resp: Response<Webhook> = self.client.get(url).send().await?.text().await?.into();
        if resp.ok {
            if let Some(ip_addr) = resp.result.ip_address {
                let state = ip_addr == ip && resp.result.has_custom_certificate;
                debug!(" webhook configured == {state}");
                return Ok(state);
            }
        }
        bail!("Could not get correct webhook");
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

    async fn get_users(&self) -> SharedUsers {
        self.users.clone()
    }
}
