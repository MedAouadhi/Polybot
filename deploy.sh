#!/bin/bash

set -eux
cross build --release --target aarch64-unknown-linux-gnu

#stop the service on the pi
ssh mohamed@awax.local sudo systemctl stop homebot.service

#copy the binary
scp target/aarch64-unknown-linux-gnu/release/homebot mohamed@awax.local:~/homebot/

#start the service
ssh mohamed@awax.local sudo systemctl start homebot.service
