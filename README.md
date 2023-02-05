# Homebot

A home server using telegram bot api, useful to respond to commands. To be used for home automation
or anything else.

There are mainly two ways to receive messages from Telegram servers, either by polling (shitty idea) using `getUpdates` API,
or by setting a `webhook` and let Telegram sends us updates. This, however, necessitates that we create our https server, which
in turn means that we need a self signed certificate, we can create it easily using: 
```
openssl req -newkey rsa:2048 -sha256 -nodes -keyout YOURPRIVATE.key -x509 -days 365 -out YOURPUBLIC.pem -subj "/C=US/ST=New York/L=Brooklyn/O=homebot Company/CN=11.11.11.11"
```
but apparently when the ip address changes, the certificate becomes invalid.
You can get away from this mess by using a static ip straight from your internet provider or using no-ip and similar services, 
because I want the cheap stuff, I want to use my own public ip address, and change the certificate everytime, the ip changes.

For this purpose, I used a hacky way with the help of `watchexec`:
- watchexec watches for changes on a special file with `.notif` extension.
- the homebot server has a background thread that has the responsaiblity of monitoring the public ip changes, and executing the `restart.sh`
script when it happens.
- the script creates the new certificate, uploads it to the Telegram servers updating the webhook, and finally touches the restart.notif file
which will trigger the cargo run. 

## Dependencies
- watchexec
- openssl


## Create the service
Create the file `/etc/systemd/system/homebot.service` with the contents
of the respective file (change the paths accordingly).
