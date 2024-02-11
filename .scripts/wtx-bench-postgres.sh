#!/usr/bin/env bash

set -euxo pipefail

RUSTFLAGS="-Ctarget-cpu=native" cargo run --bin wtx-bench --manifest-path wtx-bench/Cargo.toml --profile bench -- postgres postgres://wtx_md5:wtx@localhost:5432/wtx