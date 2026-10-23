#!/usr/bin/env bash

set -euxo pipefail

for target in wtx-fuzz/*.rs; do
  cargo fuzz run --features libfuzzer-sys/link_libfuzzer --fuzz-dir wtx-fuzz "$(basename "$target" .rs)" -- -max_total_time=15
done
