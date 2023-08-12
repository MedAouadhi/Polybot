use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_with::TimestampSeconds;
use tracing::debug;

#[derive(Deserialize)]
pub struct Affirmation {
    pub affirmation: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub server: ServerConfig,
}
#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct Chat {
    pub id: u64,
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    username: String,
    #[serde(rename(deserialize = "type"))]
    chat_type: String,
}

#[serde_with::serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct Message {
    #[serde(alias = "message_id")]
    _message_id: u64,
    #[serde(alias = "from")]
    _from: User,
    pub chat: Chat,
    #[serde_as(as = "TimestampSeconds<i64>")]
    #[serde(alias = "date")]
    _date: DateTime<Utc>,
    pub text: String,
    #[serde(skip)]
    #[serde(alias = "entities")]
    _entities: String,
}

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct User {
    id: u64,
    is_bot: bool,
    first_name: String,
    last_name: Option<String>,
    username: Option<String>,
    language_code: Option<String>,
    is_premium: Option<bool>,
    added_to_attachment_menu: Option<bool>,
    can_join_groups: Option<bool>,
    can_read_all_group_messages: Option<bool>,
    supports_inline_queries: Option<bool>,
}

#[serde_with::serde_as]
#[derive(Deserialize, Debug)]
pub struct ChatMemberUpdated {
    pub chat: Chat,
    _from: User,
    #[serde_as(as = "TimestampSeconds<i64>")]
    _date: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct Update {
    pub update_id: u64,
    pub message: Option<Message>,
    pub edited_message: Option<Message>,
    pub channel_post: Option<Message>,
    pub edited_channel_post: Option<Message>,
    /*
    #[serde(skip)]
    pub inline_query: Option<String>,
    #[serde(skip)]
    chosen_inline_result: Option<String>,
    #[serde(skip)]
    callback_query: Option<String>,
    #[serde(skip)]
    shipping_query: Option<String>,
    #[serde(skip)]
    pre_checkout_query: Option<String>,
    #[serde(skip)]
    poll: Option<String>,
    #[serde(skip)]
    poll_answer: Option<String>,
    my_chat_member: Option<ChatMemberUpdated>,
    chat_member: Option<ChatMemberUpdated>,
    #[serde(skip)]
    chat_join_request: Option<String>,
    */
}

#[derive(Deserialize, Debug)]
pub struct Response<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Deserialize)]
pub struct Webhook {
    #[serde(alias = "url")]
    _url: String,
    pub has_custom_certificate: bool,
    #[serde(alias = "pending_update_count")]
    _pending_update_count: u32,
    #[serde(alias = "max_connections")]
    _max_connections: u32,
    pub ip_address: Option<String>,
}

impl<T: for<'a> Deserialize<'a>> From<String> for Response<T> {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl From<String> for Update {
    fn from(value: String) -> Self {
        let update_str = serde_json::from_str(&value).unwrap();
        debug!("{:#?}", update_str);
        return update_str;
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
