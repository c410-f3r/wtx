#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt

pushd wtx
$rt clippy
popd
pushd wtx-examples
$rt clippy
popd
pushd wtx-ui
$rt clippy
popd

MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features http2,postgres,web-socket -p wtx --release

