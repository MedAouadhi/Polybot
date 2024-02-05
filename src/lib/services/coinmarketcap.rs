use anyhow::Ok;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Status {
    timestamp: DateTime<Utc>,
    error_code: u32,
    error_message: Option<String>,
    elapsed: u32,
    credit_count: u32,
    notice: Option<String>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct CurrencyQuote {
    price: f64,
    volume_24h: f64,
    volume_24h_reported: f64,
    volume_7d: f64,
    volume_7d_reported: f64,
    volume_30d: f64,
    volume_30d_reported: f64,
    volume_change_24h: f64,
    percent_change_1h: f64,
    percent_change_24h: f64,
    percent_change_7d: f64,
    percent_change_30d: f64,
    percent_change_60d: f64,
    percent_change_90d: f64,
    market_cap: f64,
    market_cap_dominance: f64,
    fully_diluted_market_cap: f64,
    tvl: Option<String>,
    last_updated: DateTime<Utc>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Currencies {
    #[serde(rename(deserialize = "EUR"))]
    currency: CurrencyQuote,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Data {
    id: u32,
    name: String,
    symbol: String,
    slug: String,
    date_added: DateTime<Utc>,
    circulating_supply: u32,
    infinite_supply: bool,
    self_reported_circulating_supply: Option<u32>,
    self_reported_market_cap: Option<f64>,
    tvl_ratio: Option<f64>,
    last_updated: DateTime<Utc>,
    quote: Currencies,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct CoinEntry {
    #[serde(rename(deserialize = "BTC"))]
    symbol: Vec<Data>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct BitcoinData {
    status: Status,
    data: CoinEntry,
}

/// BitcoinRequest
pub struct Coinmarket {
    api_key: String,
    client: reqwest::Client,
}

impl Coinmarket {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_bitcoin_price(&self) -> anyhow::Result<f64> {
        let mut headers = HeaderMap::new();
        headers.insert("X-CMC_PRO_API_KEY", HeaderValue::from_str(&self.api_key)?);
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        // Define the query parameters
        let params = [
            ("symbol", "BTC"),
            ("convert", "EUR"),
            ("aux", "date_added,circulating_supply,volume_24h_reported,volume_7d,volume_7d_reported,volume_30d,volume_30d_reported"),
        ];

        let resp = self
            .client
            .get("https://pro-api.coinmarketcap.com/v2/cryptocurrency/quotes/latest")
            .headers(headers)
            .query(&params)
            .send()
            .await?
            .text()
            .await?;

        let data: BitcoinData =
            serde_json::from_str(&resp).expect("problem with getting bitcoin data");

        tracing::debug!("bitcoin data: {:#?}", data);
        Ok(data.data.symbol[0].quote.currency.price)
    }
}
