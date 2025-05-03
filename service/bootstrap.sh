#!/bin/bash

sudo install -D target/debug/linux-sensor-agent /usr/local/sbin
sudo install -D service/sensor-agent.service /lib/systemd/system
sudo install -D config.toml /etc/linux-sensor-agent/config.toml
sudo systemctl daemon-reload
sudo systemctl start sensor-agent
sudo systemctl enable sensor-agent
