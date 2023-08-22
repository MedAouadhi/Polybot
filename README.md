# PolyBot

[![Rust](https://github.com/MedAouadhi/homebot/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/MedAouadhi/homebot/actions/workflows/rust.yml)

An async bot server with straightforward commands definition, with intelligent chat mode using LLMs (currently OpenAI, Llama2 in mind), and can be hosted anywhere.

## Background
The main driver for this project, was the simple idea, that I wanted to ssh to my workstation (installed in my home) from anywhere, **without paying for a static ip or a domain, and without using any 3rd party software**, I simply need the public ip address of my home network.

Well the problem is that the ip address can change at any time, so I needed a software that is running locally to the network, which publishes the ip address whenever I ask it.

Come social media (just telegram for now) bots! What is a better interface than a chat conversation in an app that I already use in my day to day life. With the bonus of adding more functionnality to the bot whenever suitable. 

I chose Rust as I am already on its learning journey, and I decided this is the perfect didactic exercise.
I initially started this as a Telegram bot server, but then to further push my trait system understanding, I decided to abstract it more, to support multiple bots (in theory).

## Features
- Async server with dynamic ip support, periodic monitoring and update of the self signed certificate of the server based on ip changes:
    - Keep all accesses local to your network, don't need a third party hosting/routing service.
    - Gives the ability to use async webhooks vs polling for the updates.
- Intuitive and simple addition of commands, simply add new function that returns a string, that's literally it.
- **Command mode** to serve back the handlers you define.
- **Chat Mode** ask your large language model, using: 
    - OpenAI models (through their API).
    - Self hosted LLM such as llama2 **(To come)**.

## Example

### Define your commands

Adding a command is as easy as annotating the handler function with the `handler(cmd = "/my_command")` attribute.
The commands need also be under a module annoted with `#[bot_commands]`.

```rust
use bot_commands_macro::{bot_commands, handler};

#[bot_commands]
pub mod commands {

    use super::*;
    use polybot::types::BotUserActions;
    use rand::Rng;

    use crate::utils::get_ip;

    #[handler(cmd = "/ip")]
    async fn ip(_user_tx: impl BotUserActions, _: String) -> String {
        if let Ok(ip) = get_ip().await {
            return ip;
        }
        "Error getting the Ip address".to_string()
    }

    #[handler(cmd = "/dice")]
    async fn dice(_: impl BotUserActions, _: String) -> String {
        rand::thread_rng().gen_range(1..=6).to_string()
    }
}

```

### Start the bot

```rust
mod bot_commands;
use anyhow::Result;
use bot_commands::commands::MyCommands;
use polybot::polybot::Polybot;
use polybot::telegram::bot::TelegramBot;
use std::error::Error;
use std::time::Duration;
use tracing::info;

// MyCommands is the macro generated struct that holds the list of commands
// defined in bot_commands.rs and implements BotCommands trait.
type MyBot = TelegramBot<MyCommands>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = polybot::utils::get_config("config.toml").await?;
    let telegrambot =
        Polybot::<MyBot>::new(config).with_webhook_monitoring(Duration::from_secs(60));

    info!("Starting Telegram Bot ...");
    telegrambot.start_loop().await?;
    Ok(())
}
```
- Note that we can opt-in/out of the webhook monitoring, which will periodically check for the validity of the self signed certificate in the
bot provider servers (e.g: Telegram), and makes sure it remains valid, by generating and uploading a new one if the ip has changed.

### Chat mode
Chat mode is simply the LLM request command (if provided) but without typing the command prefix each time, so once in the mode, you can chat with the llm just like any normal conversation.

- To inform polybot of that, we just need to tell it about the command that `starts` the chat mode, the one that `exists` it and the `llm request` command.
We do that by adding the boolean attributes in the `handler` procedural macro: 
```rust
#[handler(cmd = "/ask", llm_request = true)]
async fn ask(_user_tx: impl BotUserActions, request: String) -> String {
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
async fn chat(_user_tx: impl BotUserActions, _: String) -> String {
    "Let's chat!".to_string()
}

#[handler(cmd = "/endchat", chat_exit = true)]
async fn endchat(_user_tx: impl BotUserActions, _request: String) -> String {
    "See ya!".to_string()
}
```
## Current supported commands
- `/ip` : Gives back the current public ipv4 of the bot's network.
- `/affirm` Sends back motivational quotes.
- `/dice` Generates a random number between 1 and 6.
- `/temp [city]` Gives back the current temprature of any city in the world.
- `/ask [prompt]` Prompts the LLM agent for any single shot request.
- `/chat` Starts **chat mode** which will interpret any following messages as prompts.
- `/endchat` Exits the chat mode.


## Telegram bot example
![TelegramBot](https://github.com/MedAouadhi/Polybot/blob/master/demo.gif)

## Before you start
- Make sure to forward the port 443 in the settings of your router or firewall. For my case I forwarded all incoming requests to the port 443 to my local 4443 port.
- To make use of the llm logic, you need to run the application, with the `OPENAI_API_KEY` environment variable containing your OpenAI token, as 
PolyBot is making use of [llm-chain](https://github.com/sobelio/llm-chain).

```bash
export OPENAI_API_KEY="sk-YOUR_OPEN_AI_KEY_HERE"
```

## Configuration
1. First of all you need to create your own bot with the help of BotFather.
    - Just send a `/newbot` message to `BotFather` bot using your normal telegram account. (find more informations [here.](https://core.telegram.org/bots/tutorial)). This will give you the API token.

2. Create a `config.toml` file in the root directory of the project, with this layout:
```toml
[bot]
name = "superbot"
token = "11111111112222222222333333333"

[server]
ip = "0.0.0.0"
port = 4443
privkeyfile = "YOURPRIVATE.key"
pubkeyfile = "YOURPUBLIC.pem"
```

## Create the service
You can also create a background service to run your bot, to do that:
- Create the file `/etc/systemd/system/homebot.service` with the contents
of the [respective file](https://github.com/MedAouadhi/Polybot/blob/master/homebot.service) of this repo (change the paths accordingly).
