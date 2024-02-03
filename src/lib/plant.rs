use std::sync::Arc;
use std::time::Duration;

use crate::Bot;
use anyhow::Result;
use chrono::{DateTime, Utc};
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use rumqttc::{AsyncClient, Event};
use rumqttc::{MqttOptions, Packet};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PlantData {
    moisture: u32,
}

struct _Plant {
    name: String,
    moisture_level: u32,
    last_watering: DateTime<Utc>,
}

#[derive(InfluxDbWriteable)]
struct PlantReading {
    time: DateTime<Utc>,
    moisture: u32,
    #[influxdb(tag)]
    plant_name: String,
}

#[allow(dead_code)]
pub struct PlantServer {
    host: String,
    port: u32,
    chat_id: String,
    db_client: Client,
}

impl PlantServer {
    const MAX_DRY: u32 = 1900;
    const MIN_WET: u32 = 1500;

    pub fn new(host: &str, chat_id: &str, port: u32, db_token: &str) -> Self {
        Self {
            host: host.to_string(),
            chat_id: chat_id.to_string(),
            port,
            db_client: Client::new("http://192.168.2.132:8086", "homebucket").with_token(db_token),
        }
    }

    pub async fn start(&self, bot: Arc<impl Bot>) -> Result<()> {
        let mut avg_moisture: Vec<u32> = vec![];
        let mut mqttoptions = MqttOptions::new("homebot", "192.168.2.214", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        client
            .subscribe("plants/coleus/moisture", rumqttc::QoS::AtMostOnce)
            .await?;

        loop {
            let notification: Event = eventloop.poll().await?;

            if let Event::Incoming(Packet::Publish(data)) = notification {
                match serde_json::from_slice::<PlantData>(&data.payload) {
                    Ok(parsed_json) => {
                        let write_query = PlantReading {
                            time: Utc::now(),
                            moisture: parsed_json.moisture,
                            plant_name: "flowery".to_string(),
                        }
                        .into_query("moisture");

                        self.db_client.query(write_query).await?;

                        avg_moisture.push(parsed_json.moisture);
                        tracing::info!("Received {:?}", parsed_json);
                        if avg_moisture.len() == 12 {
                            let avg = avg_moisture.iter().sum::<u32>() / 12;
                            match avg {
                                Self::MAX_DRY.. => {
                                    // Inform the user in telegram
                                    bot.send_message(
                                        &self.chat_id,
                                        &format!("Moisture now is {}. Watering the plant!", avg),
                                    )
                                    .await?;
                                    // water the plant
                                    client
                                        .publish(
                                            "plants/coleus/water",
                                            rumqttc::QoS::AtLeastOnce,
                                            false,
                                            "true",
                                        )
                                        .await?
                                }
                                0..=Self::MIN_WET => {
                                    bot.send_message(&self.chat_id, "Coleus is too wet!")
                                        .await?;
                                    tracing::info!("Coleus is too wet!")
                                }
                                _ => (),
                            };
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
}
