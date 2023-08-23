use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::TimestampSeconds;
use tracing::debug;

use crate::types::BotMessage;

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
    pub from: User,
    pub chat: Chat,
    #[serde_as(as = "TimestampSeconds<i64>")]
    #[serde(alias = "date")]
    _date: DateTime<Utc>,
    pub text: String,
    #[serde(skip)]
    #[serde(alias = "entities")]
    _entities: String,
}

impl BotMessage for Message {
    fn get_message(&self) -> String {
        self.text.clone()
    }

    fn get_user(&self) -> (u64, String) {
        (self.from.id, self.from.first_name.clone())
    }

    fn get_chat_id(&self) -> u64 {
        self.chat.id
    }
}

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct User {
    pub id: u64,
    is_bot: bool,
    pub first_name: String,
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
        update_str
    }
}

#[derive(Serialize, Deserialize, Default)]
struct BotCommandScope {
    #[serde(rename(serialize = "type"))]
    scope_type: Scope,
    chat_id: Option<String>,
    user_id: Option<u64>,
}

#[derive(Deserialize, Default)]
pub enum Scope {
    #[default]
    BotCommandScopeDefault,
    BotCommandScopeAllPrivateChats,
    BotCommandScopeAllGroupChats,
    BotCommandScopeAllChatAdministrators,
    BotCommandScopeChat,
    BotCommandScopeChatAdministrators,
    BotCommandScopeChatMember,
}

impl serde::ser::Serialize for Scope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Scope::BotCommandScopeDefault => serializer.serialize_str("default"),
            Scope::BotCommandScopeAllPrivateChats => serializer.serialize_str("all_private_chats"),
            Scope::BotCommandScopeAllGroupChats => serializer.serialize_str("all_group_chats"),
            Scope::BotCommandScopeAllChatAdministrators => {
                serializer.serialize_str("all_chat_administrators")
            }
            Scope::BotCommandScopeChat => serializer.serialize_str("chat"),
            Scope::BotCommandScopeChatAdministrators => {
                serializer.serialize_str("chat_administrators")
            }
            Scope::BotCommandScopeChatMember => serializer.serialize_str("chat_member"),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct BotCommandsParams {
    scope: BotCommandScope,
    #[serde(skip_serializing_if = "String::is_empty")]
    language_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct BotCommandsSet {
    pub commands: Vec<BotCommand>,
    #[serde(flatten)]
    pub metadata: BotCommandsParams,
}

#[derive(Serialize, Deserialize)]
pub struct BotCommand {
    pub command: String,
    pub description: String,
}
