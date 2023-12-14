#!/usr/bin/env bash

set -euxo pipefail

cargo fuzz run --features libfuzzer-sys/link_libfuzzer --fuzz-dir wtx-fuzz web-socket -- -max_total_time=30