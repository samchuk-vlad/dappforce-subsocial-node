#!/usr/bin/env bash

cargo test --release -p df-integration-tests
cargo test --release -p pallet-utils
cargo test --release -p pallet-roles
cargo test --release -p pallet-faucets
cargo test --release -p pallet-moderation
cargo test --release -p pallet-session-keys
