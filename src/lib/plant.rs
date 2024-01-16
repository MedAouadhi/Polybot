use std::sync::Arc;

use crate::Bot;
use actix_web::cookie::time::format_description::parse;
use anyhow::Result;
use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use influxdb::{Client, Query, ReadQuery, Timestamp};
use serde::Deserialize;
use tokio::net::UdpSocket;

#[derive(Deserialize, Debug)]
pub struct PlantData {
    moisture: u32,
    is_watering: bool,
}

struct Plant {
    name: String,
    moisture_level: u32,
    last_watering: DateTime<Utc>,
}

#[derive(InfluxDbWriteable)]
struct PlantReading {
    time: DateTime<Utc>,
    moisture: u32,
    is_watering: bool,
    #[influxdb(tag)]
    plant_name: String,
}

pub struct PlantServer {
    host: String,
    port: u32,
    chat_id: String,
    db_client: Client,
}

impl PlantServer {
    pub fn new(host: &str, chat_id: &str, port: u32, db_token: &str) -> Self {
        Self {
            host: host.to_string(),
            chat_id: chat_id.to_string(),
            port,
            db_client: Client::new("http://localhost:8086", "homebucket").with_token(db_token),
        }
    }

    pub async fn start(&self, bot: Arc<impl Bot>) -> Result<()> {
        let socket = UdpSocket::bind(format!("{}:{}", self.host, self.port)).await?;
        let mut buf = [0u8; 2048];

        let mut avg_moisture: Vec<u32> = vec![];

        loop {
            let (len, _) = socket.recv_from(&mut buf).await?;
            let data = &buf[..len];

            match serde_json::from_slice::<PlantData>(data) {
                Ok(parsed_json) => {
                    let write_query = PlantReading {
                        time: Utc::now(),
                        moisture: parsed_json.moisture,
                        is_watering: parsed_json.is_watering,
                        plant_name: "flowery".to_string(),
                    }
                    .into_query("moisture");

                    self.db_client.query(write_query).await?;

                    avg_moisture.push(parsed_json.moisture);
                    tracing::info!("Received {:?}", parsed_json);
                    if avg_moisture.len() == 12 {
                        bot.send_message(
                            &self.chat_id,
                            &format!("Moisture now is {}.", avg_moisture.iter().sum::<u32>() / 12),
                        )
                        .await?;
                        avg_moisture.clear();
                    }
                }
                Err(e) => {
                    tracing::error!("Error parsing json {}", e);
                }
            }
        }
    }
}
