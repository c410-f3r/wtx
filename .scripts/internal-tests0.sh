#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt
$rt clippy -Aclippy::as_conversions,-Aclippy::modulo_arithmetic,-Aclippy::arbitrary_source_item_ordering,-Aclippy::doc-include-without-cfg,-Aclippy::little-endian-bytes,-Aclippy::panic-in-result-fn,-Aclippy::return_and_then,-Aclippy::used_underscore_items

MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features http2,postgres,web-socket -p wtx

# WTX

$rt check-generic wtx

$rt test-with-features wtx 32-tuple-impls
$rt test-with-features wtx aes-gcm
$rt test-with-features wtx arbitrary
$rt test-with-features wtx argon2
$rt test-with-features wtx base64
$rt test-with-features wtx borsh
$rt test-with-features wtx client-api-framework
$rt test-with-features wtx crossbeam-channel
$rt test-with-features wtx crypto-common
$rt test-with-features wtx database
$rt test-with-features wtx digest
$rt test-with-features wtx embassy-time
$rt test-with-features wtx executor
$rt test-with-features wtx fastrand
$rt test-with-features wtx flate2
$rt test-with-features wtx foldhash
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