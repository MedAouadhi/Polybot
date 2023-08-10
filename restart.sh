#!/bin/bash
# first argument needs to be the new ip address
# Second argument is the bot api key
set -eux
#echo "Generating new certificate and key pair"
#openssl req -newkey rsa:2048 -sha256 -nodes -keyout YOURPRIVATE.key -x509 \
#-days 365 -out YOURPUBLIC.pem -subj "/CN=$1/C=US/ST=New York/L=Brooklyn/O=homebot Company"

echo "Setting the webhook with the new certificate"
curl -F "url=https://$1/" -F "certificate=@YOURPUBLIC.pem" \
"https://api.telegram.org/bot$2/setWebhook"
# "http://0.0.0.0:80/post"
# "https://api.telegram.org/bot$2/setWebhook"

# # echo "restarting homebot service"
# sudo systemctl restart homebot.service
