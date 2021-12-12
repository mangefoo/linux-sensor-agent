#!/bin/bash

sudo cp target/debug/linux-sensor-agent /usr/local/sbin
sudo cp service/sensor-agent.service /lib/systemd/system
sudo systemctl start sensor-agent
sudo systemctl enable sensor-agent
