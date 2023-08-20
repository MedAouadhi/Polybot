use bot_commands_macro::{bot_commands, handler};

#[bot_commands]
pub mod commands {

    use super::*;
    use telegram_bot::services::llm::{Agent, OpenAiModel};
    use telegram_bot::services::openmeteo::OpenMeteo;
    use telegram_bot::types::WeatherProvider;

    use crate::utils::{get_affirmation, get_ip};

    #[handler(cmd = "/ip")]
    async fn ip(_: String) -> String {
        if let Ok(ip) = get_ip().await {
            return ip;
        }
        "Error getting the Ip address".to_string()
    }

    #[handler(cmd = "/temp")]
    async fn temp(args: String) -> String {
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
    async fn affirm(_args: String) -> String {
        if let Ok(msg) = get_affirmation().await {
            msg
        } else {
            "Problem getting the affirmation :(".into()
        }
    }

    #[handler(cmd = "/ask")]
    async fn ask(request: String) -> String {
        if request.is_empty() {
            return "Ask something!".to_string();
        }

        if let Ok(agent) = OpenAiModel::try_new() {
            if let Ok(answer) = agent.request(&request).await {
                return answer;
            }
            "Problem getting the agent response".to_string()
        } else {
            "Could not create the llm agent, check the API key".to_string()
        }
    }
}
