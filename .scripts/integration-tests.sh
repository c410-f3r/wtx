#!/usr/bin/env bash

cargo run --bin wtx-ui --features embed-migrations -- -i .test-utils/wtx.toml -o wtx/tests/embedded_migrations/mod.rs

cargo test --all-features --release -- --show-output --test-threads=1 --nocapture
