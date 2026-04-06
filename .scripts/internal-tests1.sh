#!/usr/bin/env bash

. "$(dirname "$0")/common.sh" --source-only

# WTX

$rt test-with-features wtx _async-tests
$rt test-with-features wtx _bench
$rt test-with-features wtx _hack
$rt test-with-features wtx _integration-tests
$rt test-with-features wtx _tracing-tree
$rt test-with-features wtx 32-tuple-impls
$rt test-with-features wtx aead
$rt test-with-features wtx aes-gcm
$rt test-with-features wtx arbitrary
$rt test-with-features wtx argon2
$rt test-with-features wtx asn1
$rt test-with-features wtx async-net
$rt test-with-features wtx base64
$rt test-with-features wtx chacha20
$rt test-with-features wtx chacha20poly1305
$rt test-with-features wtx client-api-framework
$rt test-with-features wtx crossbeam-channel
$rt test-with-features wtx crypto
$rt test-with-features wtx crypto-aws-lc-rs,_hack
$rt test-with-features wtx crypto-common
$rt test-with-features wtx crypto-graviola 
$rt test-with-features wtx crypto-ring
$rt test-with-features wtx crypto-rust-crypto
$rt test-with-features wtx database
$rt test-with-features wtx default
$rt test-with-features wtx digest
$rt test-with-features wtx embassy-net,_hack
# TODO: Figure out why `_embassy_time_schedule_wake` emerges even with a `std` driver
# $rt test-with-features wtx embassy-time
$rt test-with-features wtx executor
$rt test-with-features wtx fastrand
$rt test-with-features wtx flate2
$rt test-with-features wtx foldhash
$rt test-with-features wtx futures-lite
$rt test-with-features wtx getrandom