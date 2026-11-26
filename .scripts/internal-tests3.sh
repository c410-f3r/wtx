#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX

$rt test-with-features wtx serde
$rt test-with-features wtx serde_json
$rt test-with-features wtx sha3
$rt test-with-features wtx simdutf8
$rt test-with-features wtx smtp
$rt test-with-features wtx socket2
$rt test-with-features wtx std
$rt test-with-features wtx tokio
$rt test-with-features wtx tracing
$rt test-with-features wtx tracing-subscriber
$rt test-with-features wtx tracing-tree
$rt test-with-features wtx uuid
$rt test-with-features wtx web-socket,crypto-ring
$rt test-with-features wtx web-socket-server-framework
$rt test-with-features wtx web-socket,zlib-rs
$rt test-with-features wtx wtx-macros
$rt test-with-features wtx x509
$rt test-with-features wtx zlib-rs

# WTX Macros

$rt test-generic wtx-macros

# WTX UI

$rt check-generic wtx-ui
$rt test-with-features wtx-ui default
$rt test-with-features wtx-ui embed-migrations
$rt test-with-features wtx-ui schema-manager
$rt test-with-features wtx-ui schema-manager-dev
$rt test-with-features wtx-ui http-client
$rt test-with-features wtx-ui http-client,_curl
$rt test-with-features wtx-ui unified
$rt test-with-features wtx-ui web-socket
