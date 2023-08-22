use bot_commands_macro::{bot_commands, handler};

#[bot_commands]
pub mod commands {

    use super::*;
    use rand::Rng;
    use telegram_bot::services::llm::{Agent, OpenAiModel};
    use telegram_bot::services::openmeteo::OpenMeteo;
    use telegram_bot::types::{UserData, WeatherProvider};

    use crate::utils::{get_affirmation, get_ip};

    #[handler(cmd = "/ip")]
    async fn ip(_user: UserData, _: String) -> String {
        if let Ok(ip) = get_ip().await {
            return ip;
        }
        "Error getting the Ip address".to_string()
    }

    #[handler(cmd = "/temp")]
    async fn temp(_user: UserData, args: String) -> String {
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
    async fn affirm(_user: UserData, _args: String) -> String {
        if let Ok(msg) = get_affirmation().await {
            msg
        } else {
            "Problem getting the affirmation :(".into()
        }
    }

    #[handler(cmd = "/ask", llm_request = true)]
    async fn ask(_user: UserData, request: String) -> String {
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

    #[handler(cmd = "/chat", chat_start = true)]
    async fn chat(_user: UserData, _: String) -> String {
        // let users = users_m.lock().await;

        "Let's chat!".to_string()
    }

    #[handler(cmd = "/endchat", chat_exit = true)]
    async fn endchat(_user: UserData, _request: String) -> String {
        "See ya!".to_string()
    }

    #[handler(cmd = "/dice")]
    async fn dice(_: UserData, _: String) -> String {
        rand::thread_rng().gen_range(1..=6).to_string()
    }
}
