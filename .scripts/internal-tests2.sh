#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX

$rt test-with-features wtx grpc-server,crypto-ring
$rt test-with-features wtx hashbrown
$rt test-with-features wtx http
$rt test-with-features wtx http2-client-pool,crypto-ring
$rt test-with-features wtx http-cookie
$rt test-with-features wtx http-cookie-secure
$rt test-with-features wtx http-session,crypto-ring
$rt test-with-features wtx http2,crypto-ring
$rt test-with-features wtx http2-server-framework,crypto-ring
$rt test-with-features wtx httparse
$rt test-with-features wtx libc
$rt test-with-features wtx macros
$rt test-with-features wtx memchr
$rt test-with-features wtx nightly
$rt test-with-features wtx optimizations
$rt test-with-features wtx parking_lot
$rt test-with-features wtx portable-atomic
$rt test-with-features wtx portable-atomic-util
$rt test-with-features wtx postgres
$rt test-with-features wtx quick-protobuf
$rt test-with-features wtx rand_core
$rt test-with-features wtx rust_decimal
$rt test-with-features wtx schema-manager
$rt test-with-features wtx schema-manager-dev
$rt test-with-features wtx secret,crypto-ring