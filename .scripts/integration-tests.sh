#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

export DATABASE_URI='postgres://wtx:wtx@localhost:5432/wtx'

$rt test-with-features wtx _integration-tests,database,sm-dev

cargo run --bin wtx-ui --features embed-migrations -- embed-migrations -i .test-utils/migrations.toml -o wtx/tests/embedded_migrations/mod.rs 