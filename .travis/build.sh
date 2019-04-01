#!/usr/bin/env bash
cd /liftinstall || exit 1

apt-get update
apt-get install -y libwebkit2gtk-4.0-dev libssl-dev

curl https://build.travis-ci.org/files/rustup-init.sh -sSf | sh -s -- -y
export PATH=~/.cargo/bin:$PATH

cargo build
