#!/usr/bin/env bash

set -e

echo "It will take a long time. Project will be built twice..."

cargo build --release

./target/release/subsocial-node build-spec --disable-default-bootnode --chain staging > ./node/res/customSpec.json
./target/release/subsocial-node build-spec --chain=./node/res/customSpec.json --raw --disable-default-bootnode > ./node/res/subsocialSpecJson.json

cargo build --release
