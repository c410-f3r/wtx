#!/usr/bin/env bash

export DATABASE_URI_MYSQL='mysql://wtx:wtx@localhost:3306/wtx'
export DATABASE_URI_POSTGRES='postgres://wtx_scram:wtx@localhost:5432/wtx'

cargo run --bin wtx-ui --features embed-migrations -- -i .test-utils/migrations.toml -o wtx/tests/embedded_migrations/mod.rs

cargo test --all-features --release -- --show-output --test-threads=1 --nocapture
