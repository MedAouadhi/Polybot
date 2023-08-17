use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Affirmation {
    pub affirmation: String,
}
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
