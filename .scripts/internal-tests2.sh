#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX

$rt test-with-features wtx grpc
$rt test-with-features wtx grpc-client
$rt test-with-features wtx grpc-server
$rt test-with-features wtx hashbrown
$rt test-with-features wtx hkdf
$rt test-with-features wtx hmac
$rt test-with-features wtx http
$rt test-with-features wtx http-client-pool
$rt test-with-features wtx http-cookie
$rt test-with-features wtx http-cookie-secure
$rt test-with-features wtx http-server-framework
$rt test-with-features wtx http-session
$rt test-with-features wtx http2
$rt test-with-features wtx httparse
$rt test-with-features wtx libc
$rt test-with-features wtx macros
$rt test-with-features wtx matchit
$rt test-with-features wtx memchr
$rt test-with-features wtx mysql
$rt test-with-features wtx nightly
$rt test-with-features wtx optimization
$rt test-with-features wtx parking_lot
$rt test-with-features wtx portable-atomic
$rt test-with-features wtx portable-atomic-util
$rt test-with-features wtx postgres
$rt test-with-features wtx quick-protobuf
$rt test-with-features wtx rand_core
$rt test-with-features wtx rand-compat
$rt test-with-features wtx ring
$rt test-with-features wtx rsa