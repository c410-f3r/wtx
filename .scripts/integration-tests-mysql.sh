#!/usr/bin/env bash

cargo test --features _async-tests,mysql,_integration-tests --package wtx --release -- --test-threads=1
