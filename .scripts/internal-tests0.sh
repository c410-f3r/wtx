#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt
$rt clippy -Aclippy::as_conversions,-Aclippy::cfg_not_test,-Aclippy::modulo_arithmetic,-Aclippy::arbitrary_source_item_ordering,-Aclippy::doc-include-without-cfg,-Aclippy::little-endian-bytes,-Aclippy::panic-in-result-fn,-Aclippy::return_and_then,-Aclippy::used_underscore_items

MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --features http2,postgres,web-socket -p wtx

# WTX

$rt check-generic wtx

$rt test-with-features wtx _async-tests
$rt test-with-features wtx _bench
$rt test-with-features wtx _integration-tests
$rt test-with-features wtx _tracing-tree
$rt test-with-features wtx 32-tuple-impls
$rt test-with-features wtx aes-gcm
$rt test-with-features wtx arbitrary
$rt test-with-features wtx argon2
$rt test-with-features wtx async-net
$rt test-with-features wtx base64
$rt test-with-features wtx borsh
$rt test-with-features wtx chacha20
$rt test-with-features wtx client-api-framework
$rt test-with-features wtx crossbeam-channel
$rt test-with-features wtx crypto-common
$rt test-with-features wtx database
