use anyhow::{Context, Result};
use polybot::Config;

use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::env;
use tokio::fs::{self};
use tracing::debug;

pub async fn get_config() -> Result<Config> {
    let mut config_file = env::current_dir().unwrap();
    config_file.push("config.toml");
    let toml_str = fs::read_to_string(config_file)
        .await
        .expect("Missing 'config.toml' file!");
    let map: Config = toml::from_str(&toml_str)?;
    debug!("{:#?}", map);
    Ok(map)
}

#[derive(Deserialize)]
struct Ipify {
    ip: String,
}
pub async fn get_ip() -> Result<String> {
    let resp: String = reqwest::Client::new()
        .get("https://api.ipify.org?format=json")
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?
        .text()
        .await?;
    let result: Ipify = serde_json::from_str(&resp).context("Failed to get the ip address")?;
    Ok(result.ip)
}

#[derive(Deserialize)]
pub struct Affirmation {
    pub affirmation: String,
}
pub async fn get_affirmation() -> Result<String> {
    let url = "https://affirmations.dev".to_string();
    let resp = reqwest::Client::new()
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let text: Affirmation = serde_json::from_str(&resp).unwrap();
    Ok(text.affirmation)
}
#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_get_config() {
        // This test assumes that a valid 'config.toml' file is present in the current directory.
        // Create a dummy 'config.toml' file in a temp directory
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let data = toml::toml! {
            [bot]
            name = "dummy"
            token = "dummytoken"

            [server]
            ip = "0.0.0.0"
            port = 4443
            privkeyfile = "YOURPRIVATE.key"
            pubkeyfile = "YOURPUBLIC.pem"
        };

        fs::write(&config_path, data.to_string()).await.unwrap();
        let current_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = get_config().await;
        println!("{:#?}", result.as_ref().unwrap());
        // Reset the current directory back to what it was
        std::env::set_current_dir(current_dir).unwrap();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_affirmation() {
        // Mock the affirmations API
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(httpmock::Method::GET);
            then.status(200)
                .body(r#"{ "affirmation": "You are awesome!" }"#);
        });

        let resp = reqwest::Client::new()
            .get(&server.url("/"))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let text: Affirmation = serde_json::from_str(&resp).unwrap();

        mock.assert();
        assert_eq!(text.affirmation, "You are awesome!");
    }
}
