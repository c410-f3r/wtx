#!/usr/bin/env bash

export DATABASE_URI_MYSQL='mysql://wtx:wtx@localhost:3307/wtx'

cargo test --features _async-tests,mysql,_integration-tests --package wtx --release -- --test-threads=1

export DATABASE_URI_MYSQL='mysql://wtx:wtx@localhost:3308/wtx'

cargo test --features _async-tests,mysql,_integration-tests --package wtx --release -- --test-threads=1
