use bot_commands_macro::bot_commands;

#[bot_commands]
pub mod commands {

    use super::*;
    use crate::openmeteo::OpenMeteo;
    use crate::types::Bot;

    use crate::utils::{get_affirmation, get_ip};

    #[handler(cmd = "/ip")]
    async fn get_ip_handler(_bot: &impl Bot, _: &str) -> String {
        if let Ok(ip) = get_ip().await {
            return ip;
        }
        "Error getting the Ip address".to_string()
    }

    #[handler(cmd = "/temp")]
    async fn get_temp(_: &impl Bot, args: &str) -> String {
        use crate::types::WeatherProvider;
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

    #[handler(cmd = "/affirm")]
    async fn affirm(_: &impl Bot, _args: &str) -> String {
        if let Ok(msg) = get_affirmation().await {
            msg
        } else {
            "Problem getting the affirmation :(".into()
        }
    }
}
