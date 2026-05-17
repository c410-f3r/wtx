#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt
$rt clippy

MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features http2,postgres,web-socket -p wtx

