#!/usr/bin/env bash

curl -o /tmp/ktls.h https://raw.githubusercontent.com/torvalds/linux/master/include/uapi/linux/tls.h
bindgen /tmp/ktls.h -o wtx/src/tls/ktls_bindings.rs