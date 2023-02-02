use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct Chat {
    pub id: u64,
    first_name: String,
    last_name: String,
    username: String,
    #[serde(rename(deserialize = "type"))]
    chat_type: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Message {
    message_id: u64,
    #[serde(skip)]
    from: String,
    pub chat: Chat,
    date: u64,
    pub text: String,
    #[serde(skip)]
    entities: String,
}

#[derive(Deserialize)]
pub struct Update {
    pub update_id: u64,
    pub message: Message,
}

#[derive(Deserialize)]
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
        serde_json::from_str(&value).unwrap()
    }
}
