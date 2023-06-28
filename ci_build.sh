#!/bin/bash

apt update -y
apt install -y libudev-dev

cargo build --release

wget -qO- https://github.com/thedodd/trunk/releases/download/v0.17.0/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-
trunk build --release
