#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

export DATABASE_URI='postgres://wtx_md5:wtx@localhost:5432/wtx'
export RUST_LOG=info

cargo test --all-features --release -- --test-threads=1

cargo run --bin wtx-ui --features embed-migrations -- embed-migrations -i .test-utils/migrations.toml -o wtx/tests/embedded_migrations/mod.rs 