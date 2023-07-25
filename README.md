# Homebot

[![Rust](https://github.com/MedAouadhi/homebot/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/MedAouadhi/homebot/actions/workflows/rust.yml)

A home server using telegram bot api, useful to respond to commands. To be used for home automation
or anything else.

There are mainly two ways to receive messages from Telegram servers, either by polling (shitty idea) using `getUpdates` API,
or by setting a `webhook` and let Telegram sends us updates. This, however, necessitates that we create our https server, which in turn means that we need a self signed certificate, we can create it easily using: 
```
openssl req -newkey rsa:2048 -sha256 -nodes -keyout YOURPRIVATE.key -x509 -days 365 -out YOURPUBLIC.pem -subj "/C=US/ST=New York/L=Brooklyn/O=homebot Company/CN=11.11.11.11"
```
but apparently when the ip address changes, the certificate becomes invalid.
You can get away from this mess by using a static ip straight from your internet provider or using no-ip and similar services.

Because I want the cheap stuff, I want to use my own public ip address, and change the certificate everytime, the ip changes.

For this purpose, I used a hacky way with the help of a simple script:

- In the main function, we spawn a thread that will check each minute for changes, between our current ip and the the one the current
webhook is configured with. The thread will start a the bash script `restart.sh` in case of change.
- the script creates the new certificate, uploads it to the Telegram servers updating the webhook, and restarts the homebot service, which will restart the application again.

## Before you start
Make sure to forward the port 443 in the settings of your router or firewall. For my case I forwarded all incoming requests to the port 443 to my local 4443 port.

## Configuration
1. First of all you need to create your own bot with the help of BotFather.
    - Just send a `/newbot` message to `BotFather` bot using your normal telegram account. (find more informations [here.](https://core.telegram.org/bots/tutorial)). This will give the API key that you will need later.

2. Create a `config.toml` file in the root directory of the project, with this layout:
```toml
[bot]
name = "superbot"
token = "11111111112222222222333333333"
```
3. For the first time only:
execute the `restart.sh` script with the correct ip address and token, e.g:
```
/home/$USER/homebot/restart.sh 1.2.3.4 "token1111122222"
```
## Dependencies
- openssl

## Create the service
Create the file `/etc/systemd/system/homebot.service` with the contents
of the respective file (change the paths accordingly).
