#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX

$rt test-with-features wtx digest
$rt test-with-features wtx embassy-time
$rt test-with-features wtx executor
$rt test-with-features wtx fastrand
$rt test-with-features wtx flate2
$rt test-with-features wtx foldhash
$rt test-with-features wtx futures-lite
$rt test-with-features wtx grpc
$rt test-with-features wtx grpc-client
$rt test-with-features wtx grpc-server
$rt test-with-features wtx hashbrown
$rt test-with-features wtx hmac
$rt test-with-features wtx http-client-pool
$rt test-with-features wtx http-cookie
$rt test-with-features wtx http-cookie-secure
$rt test-with-features wtx http-server-framework
$rt test-with-features wtx http-session
$rt test-with-features wtx http2
$rt test-with-features wtx httparse
$rt test-with-features wtx matchit
$rt test-with-features wtx memchr
$rt test-with-features wtx mysql
$rt test-with-features wtx nightly
$rt test-with-features wtx optimization
$rt test-with-features wtx pool
$rt test-with-features wtx portable-atomic-util
$rt test-with-features wtx postgres
$rt test-with-features wtx quick-protobuf
$rt test-with-features wtx rand_chacha
$rt test-with-features wtx ring
$rt test-with-features wtx rust_decimal
$rt test-with-features wtx rustls
$rt test-with-features wtx schema-manager