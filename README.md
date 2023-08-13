# Homebot

[![Rust](https://github.com/MedAouadhi/homebot/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/MedAouadhi/homebot/actions/workflows/rust.yml)

A bot server using telegram bot api, useful to respond to custom commands. To be used for home automation
or anything your mind.

## Current supported commands
- `/ip` : Gives back the current public ipv4 of the bot's network.
- `/affirm` Sends back motivational quotes.
- `/dice` Generates a random number between 1 and 6.
- `/temp [city]` Gives back the current temprature of any city in the world.

## Description
There are mainly two ways to receive messages from Telegram servers, either by polling (shitty idea) using `getUpdates` API,
or by setting a `webhook` and let Telegram sends us updates. This, however, necessitates that we create our https server, which in turn means that we need a self signed certificate, we can create it easily using: 
```
openssl req -newkey rsa:2048 -sha256 -nodes -keyout YOURPRIVATE.key -x509 -days 365 -out YOURPUBLIC.pem -subj "/C=US/ST=New York/L=Brooklyn/O=homebot Company/CN=11.11.11.11"
```
---
**Note:** The bot can generate its own certificates now. No need to generate it manually.

---
but apparently when the ip address changes, the certificate becomes invalid.
You can get away from this mess by using a static ip straight from your internet provider or using no-ip and similar services.

Because I want the cheap stuff, I want to use my own public ip address. 

To do that, we spawn an async task along with the main loop to check if the current ip of the bot and the one (the CN field)
in the uploaded certificate match, if not (most likely your router has restarted, or you moved the bot to another place), it will generate
a new self signed certificate using the new ip, and uploads it to Telegram updating the webhook.

## Before you start
Make sure to forward the port 443 in the settings of your router or firewall. For my case I forwarded all incoming requests to the port 443 to my local 4443 port.

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
## Dependencies
- openssl

## Create the service
You can also create a background service to run your bot, to do that:
- Create the file `/etc/systemd/system/homebot.service` with the contents
of the respective file of this repo (change the paths accordingly).
