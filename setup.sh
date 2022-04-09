#!/bin/bash
echo "setup ubuntu souce"
cat /host/etc/source.list >> /etc/apt/sources.list

echo "update"
apt-get update
apt-get -y upgrade

echo "config tun module"
modprobe tun

echo "config rustup and cargo"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source {$HOME}/.cargo/env

echo "set nighty"
rustup override set nightly

echo "install deps"
apt-get install pkg-config -y
apt-get install libssl-dev -y
