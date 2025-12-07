#!/usr/bin/env bash

cd wtx
LOOM_MAX_PREEMPTIONS=2 \
LOOM_MAX_BRANCHES=1000000 \
RUSTFLAGS="--cfg loom" \
cargo test --features loom --profile deploy loom_tests