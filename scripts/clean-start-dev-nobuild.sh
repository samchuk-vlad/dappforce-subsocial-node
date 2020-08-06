#!/usr/bin/env bash

set -e

./target/release/subsocial-node purge-chain --dev -y
./target/release/subsocial-node --dev
