use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Chat {
    pub id: u64,
    first_name: String,
    last_name: String,
    username: String,
    #[serde(rename(deserialize = "type"))]
    chatType: String,
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
pub struct Response {
    pub ok: bool,
    pub result: Vec<Update>,
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}
