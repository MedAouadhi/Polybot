use serde::Deserialize;
use serde_with::TimestampSeconds;
use chrono::{DateTime, Utc};

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
    message_id: u64,
    from: User,
    pub chat: Chat,
    #[serde_as(as = "TimestampSeconds<i64>")]
    date: DateTime<Utc>,
    pub text: String,
    #[serde(skip)]
    entities: String,
}

#[derive(Deserialize, Clone, Debug)]
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
    from: User,
    #[serde_as(as = "TimestampSeconds<i64>")]
    date: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct Update {
    pub update_id: u64,
    pub message: Option<Message>,
    pub edited_message: Option<Message>,
    pub channel_post: Option<Message>,
    pub edited_channel_post: Option<Message>, 
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
}

#[derive(Deserialize, Debug)]
pub struct Response<T> {
    pub ok: bool,
    pub result: T,
}

#[derive(Deserialize)]
pub struct Webhook {
    url: String,
    has_custom_certificate: bool,
    pending_update_count: u32,
    max_connections: u32,
    pub ip_address: String,
}

impl<T: for<'a> Deserialize<'a>> From<String> for Response<T> {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}

impl From<String> for Update {
    fn from(value: String) -> Self {
        let update_str = serde_json::from_str(&value).unwrap();
        println!("{:#?}", update_str);
        return update_str;
    }
}
