#!/usr/bin/env bash

export DATABASE_URI='postgres://wtx_scram:wtx@localhost:5432/wtx'

cargo test --all-features --release -- --test-threads=1

cargo run --bin wtx-ui --features embed-migrations -- -i .test-utils/migrations.toml -o wtx/tests/embedded_migrations/mod.rs 