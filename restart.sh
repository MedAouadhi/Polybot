#!/bin/bash
# first argument needs to be the new ip address
# Second argument is the bot api key
set -eux
echo "Generating new certificate and key pair"
openssl req -newkey rsa:2048 -sha256 -nodes -keyout YOURPRIVATE.key -x509 \
-days 365 -out YOURPUBLIC.pem -subj "/C=US/ST=New York/L=Brooklyn/O=homebot Company/CN=$1"

echo "Setting the webhook with the new certificate"
curl -F "url=https://$1/" -F "certificate=@YOURPUBLIC.pem" \
"https://api.telegram.org/bot$2/setWebhook"


echo "Using watchexec restart the homebot by updating the notif file"
echo $(date) >> restart.notif