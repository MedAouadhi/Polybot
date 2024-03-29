use crate::types::{ForecastTime, WeatherProvider};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Timelike;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct HourlyUnits {
    #[serde(alias = "time")]
    _time: String,
    #[serde(alias = "temperature_2m")]
    _temperature_2m: String,
}

#[derive(Deserialize, Debug)]
struct Hourly {
    #[serde(alias = "time")]
    _time: Vec<String>,
    temperature_2m: Vec<f32>,
}

#[derive(Deserialize, Debug)]
struct Forecast {
    #[serde(alias = "latitude")]
    _latitude: f32,
    #[serde(alias = "longitude")]
    _longitude: f32,
    #[serde(alias = "generationtime_ms")]
    _generationtime_ms: f32,
    #[serde(alias = "utc_offset_seconds")]
    _utc_offset_seconds: u32,
    #[serde(alias = "timezone")]
    _timezone: String,
    #[serde(alias = "timezone_abbreviation")]
    _timezone_abbreviation: String,
    #[serde(skip)]
    _elevation: f32,
    #[serde(alias = "hourly_units")]
    _hourly_units: HourlyUnits,
    hourly: Hourly,
}

#[derive(Deserialize, Debug)]
struct City {
    #[serde(alias = "id")]
    _id: u32,
    #[serde(alias = "name")]
    _name: String,
    latitude: f32,
    longitude: f32,
    #[serde(skip)]
    _elevation: f32,
    #[serde(skip)]
    _feature_code: String,
    #[serde(skip)]
    _country_code: String,
    #[serde(skip)]
    _admin1_id: u32,
    #[serde(skip)]
    _admin3_id: u32,
    #[serde(skip)]
    _admin4_id: u32,
    #[serde(alias = "timezone")]
    _timezone: String,
    #[serde(skip)]
    _population: u32,
    #[serde(skip)]
    _postcodes: Vec<String>,
    #[serde(skip)]
    _country_id: u32,
    #[serde(skip)]
    _country: String,
    #[serde(skip)]
    _admin1: String,
    #[serde(skip)]
    _admin3: String,
    #[serde(skip)]
    _admin4: String,
}

#[derive(Deserialize, Debug)]
struct Geolocation {
    results: Option<Vec<City>>,
    #[serde(alias = "generationtime_ms")]
    _generationtime_ms: f32,
}

#[derive(Clone, Default)]
pub struct OpenMeteo {
    _api_key: Option<String>,
    client: reqwest::Client,
    favourite_city: String,
}

impl OpenMeteo {
    pub fn new(api_key: Option<String>, default_city: String) -> Self {
        Self {
            _api_key: api_key,
            client: reqwest::Client::new(),
            favourite_city: default_city,
        }
    }

    async fn get_geolocation(&self, city: String) -> Result<Option<(f32, f32)>> {
        let resp = self
            .client
            .get(format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json", city))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?
            .text()
            .await?;

        let data: Geolocation =
            serde_json::from_str(&resp).expect("problem with getting geolocation data");

        if let Some(results) = data.results {
            Ok(Some((results[0].latitude, results[0].longitude)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn get_forecast_url(lat: f32, long: f32, days: u32) -> String {
        format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&forecast_days={}", lat, long, days)
    }
}

#[async_trait]
impl WeatherProvider for OpenMeteo {
    async fn get_temperature(&self, city: String) -> Option<f32> {
        if let Ok(Some((lat, long))) = self.get_geolocation(city).await {
            let resp = if let Ok(req) = self
                .client
                .get(OpenMeteo::get_forecast_url(lat, long, 1))
                .header(CONTENT_TYPE, "application/json")
                .send()
                .await
            {
                req.text().await.ok()
            } else {
                return None;
            };

            if let Some(data) = resp {
                let hour = chrono::Local::now().hour();
                let forecast: Forecast = serde_json::from_str(&data).unwrap();
                return Some(forecast.hourly.temperature_2m[hour as usize]);
            }
        }
        None
    }

    async fn get_temp_forecast(&self, _city: String, _time: ForecastTime) -> Option<f32> {
        todo!()
    }

    fn get_favourite_city(&self) -> String {
        self.favourite_city.clone()
    }
}

#[cfg(test)]
mod test {
    use crate::{services::openmeteo::OpenMeteo, types::WeatherProvider};

    #[tokio::test]
    async fn test_get_geolocation() {
        let weather = OpenMeteo::new(None, "Berlin".to_string());
        assert_eq!(
            weather.get_geolocation("Bizerte".to_string()).await.ok(),
            Some(Some((37.27442f32, 9.87391f32)))
        );
        assert_eq!(
            weather.get_geolocation("Zagreb".to_string()).await.ok(),
            Some(Some((45.81444f32, 15.97798f32)))
        );
        assert_eq!(
            weather.get_geolocation("Lalaland".to_string()).await.ok(),
            Some(None)
        );
    }

    #[tokio::test]
    async fn test_get_temperature() {
        let weather = OpenMeteo::default();
        let temp = weather
            .get_temperature("Bizerte".to_string())
            .await
            .unwrap();
        // This test will fail due to either something breaking, or
        // to global warming!
        assert!(temp > -10.0f32);
        assert!(temp < 50.0f32);
        assert_eq!(weather.get_temperature("Mordor".to_string()).await, None);
    }
}
