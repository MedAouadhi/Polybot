use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use crate::telegram::types::Message;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use enum_dispatch::enum_dispatch;
use llm_chain::chains::conversation::Chain;
use llm_chain::prompt;
use serde::Deserialize;
use tokio::sync::Mutex;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub server: ServerConfig,
}
#[derive(Deserialize, Debug, Clone, Default)]
pub struct BotConfig {
    pub name: String,
    pub token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u32,
    #[serde(alias = "pubkeyfile")]
    pub pubkey_path: String,
    #[serde(alias = "privkeyfile")]
    pub privkey_path: String,
}

#[enum_dispatch]
pub trait BotMessage {
    fn get_message(&self) -> String;
    fn get_user(&self) -> (u64, String);
    fn get_chat_id(&self) -> u64;
}

/// Here we can fill out all of the implementors of Bot and their respective
/// message types.
#[enum_dispatch(BotMessage)]
pub enum BotMessages {
    Message, // Telegram messages
}

pub type SharedUser = Arc<RwLock<BotUser>>;
pub type SharedUsers = Arc<Mutex<HashMap<u64, SharedUser>>>;
pub type CommandHashMap = HashMap<String, Box<dyn BotCommandHandler + Send + Sync>>;

#[async_trait]
pub trait Bot: Send + Sync + 'static {
    async fn initialize(&self) -> Result<()>;
    async fn handle_message(&self, msg: String) -> Result<()>;
    async fn is_webhook_configured(&self, ip: &str) -> Result<bool>;
    async fn update_webhook_cert(&self, cert: PathBuf, ip: &str) -> Result<()>;
    fn get_webhook_ips(&self) -> Result<Vec<&'static str>>;
    fn new(config: BotConfig) -> Self
    where
        Self: Sized;
}

pub trait BotCommands: Default + Send + Sync {
    fn command_list() -> CommandHashMap;
    fn chat_start_command() -> Option<&'static str>;
    fn chat_exit_command() -> Option<&'static str>;
    fn llm_request_command() -> Option<&'static str>;
}

#[async_trait]
pub trait BotCommandHandler {
    async fn handle(&self, user: SharedUser, args: String) -> String;
}

#[derive(Default)]
pub struct BotUser {
    chat_mode: AtomicBool,
    last_activity: DateTime<Utc>,
    // chain: Arc<RwLock<Chain>>,
    chain: Arc<Mutex<Chain>>,
}

impl BotUser {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
pub trait BotUserActions {
    async fn set_last_activity(&mut self, date: DateTime<Utc>);
    async fn get_last_activity(&self) -> DateTime<Utc>;
    async fn set_chat_mode(&self, state: bool);
    async fn is_in_chat_mode(&self) -> bool;
    async fn get_conversation(&self) -> Arc<Mutex<Chain>>;
    async fn reset_conversation_chain(&self, system_prompt: &str) -> Result<()>;
}

#[async_trait]
impl BotUserActions for SharedUser {
    async fn set_last_activity(&mut self, date: DateTime<Utc>) {
        self.write().expect("poisoned lock").last_activity = date;
    }

    async fn get_last_activity(&self) -> DateTime<Utc> {
        self.read().expect("poisoned lock").last_activity
    }

    async fn set_chat_mode(&self, state: bool) {
        self.write()
            .expect("poisoned lock")
            .chat_mode
            .store(state, Ordering::Relaxed);
    }

    async fn is_in_chat_mode(&self) -> bool {
        self.read()
            .expect("poisoned lock")
            .chat_mode
            .load(Ordering::Relaxed)
    }

    async fn get_conversation(&self) -> Arc<Mutex<Chain>> {
        self.read().expect("poisoned lock").chain.clone()
    }

    async fn reset_conversation_chain(&self, system_prompt: &str) -> Result<()> {
        self.write().expect("poisoned lock").chain =
            Arc::new(Mutex::new(Chain::new(prompt!(system: system_prompt))?));
        Ok(())
    }
}

pub enum ForecastTime {
    Later(u32),
    Tomorrow,
}

#[async_trait]
pub trait WeatherProvider: Sync + Send + Clone {
    async fn get_temperature(&self, city: String) -> Option<f32>;
    async fn get_temp_forecast(&self, city: String, time: ForecastTime) -> Option<f32>;
    fn get_favourite_city(&self) -> String;
}
