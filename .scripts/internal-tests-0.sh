#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

$rt rustfmt
$rt clippy -Aclippy::unneeded_field_pattern,-Aclippy::enum_variant_names,-Aclippy::else_if_without_else

cargo miri test -p wtx

# WTX

$rt check-generic wtx
$rt test-with-features wtx ahash
$rt test-with-features wtx arbitrary
$rt test-with-features wtx atoi
$rt test-with-features wtx base64
$rt test-with-features wtx borsh
$rt test-with-features wtx chrono
$rt test-with-features wtx cl-aux
$rt test-with-features wtx client-api-framework
$rt test-with-features wtx crypto-common
$rt test-with-features wtx data-transformation
$rt test-with-features wtx database
$rt test-with-features wtx digest
$rt test-with-features wtx fastrand
$rt test-with-features wtx flate2
$rt test-with-features wtx grpc
$rt test-with-features wtx hashbrown
$rt test-with-features wtx hmac
$rt test-with-features wtx http-client-framework
$rt test-with-features wtx http-server-framework
$rt test-with-features wtx http2
$rt test-with-features wtx httparse
$rt test-with-features wtx memchr
$rt test-with-features wtx pool
$rt test-with-features wtx postgres
$rt test-with-features wtx proptest
$rt test-with-features wtx quick-protobuf
$rt test-with-features wtx rand
$rt test-with-features wtx ring
$rt test-with-features wtx rkyv,_hack
$rt test-with-features wtx rust_decimal
$rt test-with-features wtx schema-manager
$rt test-with-features wtx schema-manager-dev
$rt test-with-features wtx serde
$rt test-with-features wtx serde_json
$rt test-with-features wtx sha1
$rt test-with-features wtx sha2
$rt test-with-features wtx simdutf8
$rt test-with-features wtx std
$rt test-with-features wtx test-strategy
$rt test-with-features wtx tokio
$rt test-with-features wtx tokio-rustls
$rt test-with-features wtx tracing
$rt test-with-features wtx web-socket
$rt test-with-features wtx web-socket-handshake
$rt test-with-features wtx webpki-roots
$rt test-with-features wtx x509-certificate

$rt check-with-features wtx _bench
$rt check-with-features wtx _hack
$rt check-with-features wtx _integration-tests
$rt check-with-features wtx _tracing-tree
$rt test-with-features wtx _proptest

# WTX Macros

$rt test-generic wtx-macros

# WTX UI

$rt check-generic wtx-ui
$rt test-with-features wtx-ui embed-migrations
$rt test-with-features wtx-ui schema-manager
$rt test-with-features wtx-ui schema-manager-dev
$rt test-with-features wtx-ui http-client
$rt test-with-features wtx-ui web-socket