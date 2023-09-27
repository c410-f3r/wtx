#!/usr/bin/env bash

set -euxo pipefail

cargo fuzz run --features="libfuzzer-sys/link_libfuzzer" --fuzz-dir wtx-fuzz parse-frame -- -max_total_time=30
cargo fuzz run --features="libfuzzer-sys/link_libfuzzer" --fuzz-dir wtx-fuzz unmask -- -max_total_time=30
