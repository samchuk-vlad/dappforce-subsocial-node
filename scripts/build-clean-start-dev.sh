#!/usr/bin/env bash

set -e

cargo build --release
./target/release/subsocial-node purge-chain --dev -y
./target/release/subsocial-node --dev
