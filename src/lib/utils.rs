use crate::Config;
use anyhow::{Context, Result};

use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    pkey::PKey,
    rsa::Rsa,
    x509::{X509NameBuilder, X509},
};
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::{env, path::PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tracing::info;

pub async fn get_config(file: &str) -> Result<Config> {
    let mut config_file = env::current_dir().unwrap();
    config_file.push(file);
    let toml_str = fs::read_to_string(config_file)
        .await
        .expect("Missing 'config.toml' file!");
    let map: Config = toml::from_str(&toml_str)?;
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

pub async fn generate_certificate(
    pubkey: PathBuf,
    privkey: PathBuf,
    ip: &str,
    name: &str,
) -> Result<()> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "DE")?;
    x509_name.append_entry_by_text("ST", "B")?;
    x509_name.append_entry_by_text("O", name)?;
    x509_name.append_entry_by_text("CN", ip)?;
    let x509_name = x509_name.build();

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&key_pair)?;

    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;

    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.sign(&key_pair, openssl::hash::MessageDigest::sha256())?;
    let cert = cert_builder.build();

    fs::write(&pubkey, cert.to_pem()?).await?;
    fs::write(&privkey, &key_pair.private_key_to_pem_pkcs8()?).await?;

    let mut file = File::open(&pubkey).await?;
    file.flush().await?;
    file.sync_all().await?;

    let mut file = File::open(&privkey).await?;
    file.flush().await?;
    file.sync_all().await?;

    info!("Generated the keys !");
    Ok(())
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

        let result = get_config("config.toml").await;
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
