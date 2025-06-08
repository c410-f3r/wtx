#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX

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
$rt test-with-features wtx schema-manager-dev
$rt test-with-features wtx serde
$rt test-with-features wtx serde_json
$rt test-with-features wtx sha1
$rt test-with-features wtx sha2
$rt test-with-features wtx simdutf8
$rt test-with-features wtx std
$rt test-with-features wtx tokio
$rt test-with-features wtx tokio-rustls
$rt test-with-features wtx tracing
$rt test-with-features wtx uuid
$rt test-with-features wtx web-socket
$rt test-with-features wtx web-socket-handshake
$rt test-with-features wtx webpki-roots

$rt test-with-features wtx _async-tests
$rt test-with-features wtx _bench
$rt test-with-features wtx _integration-tests
$rt test-with-features wtx _tracing-tree

# WTX Macros

$rt test-generic wtx-macros

# WTX UI

$rt check-generic wtx-ui
$rt test-with-features wtx-ui embed-migrations
$rt test-with-features wtx-ui schema-manager
$rt test-with-features wtx-ui schema-manager-dev
$rt test-with-features wtx-ui http-client
$rt test-with-features wtx-ui web-socket
