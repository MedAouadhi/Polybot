use crate::utils::get_ip;
use bot_commands_macro::bot_commands;

#[bot_commands]
pub mod commands {

    use super::super::bot::Bot;
    use super::*;
    use crate::openmeteo::OpenMeteo;
    use crate::types::WeatherProvider;

    #[handler(cmd = "/ip")]
    pub async fn get_ip_handler(_bot: &impl Bot, _: &str) -> String {
        if let Ok(ip) = get_ip().await {
            return ip;
        }
        "Error getting the Ip address".to_string()
    }

    #[handler(cmd = "/temp")]
    pub async fn get_temp(_: &impl Bot, args: &str) -> String {
        let weather = OpenMeteo::new(None, "Lehnitz".to_string());
        let mut city = weather.get_favourite_city();
        if !args.is_empty() {
            city = args.to_string();
        }
        if let Some(temp) = weather.get_temperature(city).await {
            temp.to_string()
        } else {
            "Error getting the temp".into()
        }
    }
}
