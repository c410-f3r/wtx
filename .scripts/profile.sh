#!/usr/bin/env bash

# sudo setcap 'cap_perfmon+ep' `which samply`
# sudo sysctl kernel.perf_event_mlock_kb=2048

CMD="cargo test --profile profiling --no-run"

BINARY=$($CMD -v 2>&1  | grep -o "\`.*\`")

samply record ${BINARY:1:-1}
