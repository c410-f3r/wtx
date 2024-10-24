# WebSocket

Implementation of [RFC6455](https://datatracker.ietf.org/doc/html/rfc6455) and [RFC7692](https://datatracker.ietf.org/doc/html/rfc7692).

To use this functionality, it necessary to activate the `web-socket` feature.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

## Autobahn Reports

1. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingclient/index.html" target="_blank">fuzzingclient</a>
2. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingserver/index.html" target="_blank">fuzzingserver</a>

## Compression

The "permessage-deflate" extension is the only supported compression format and is backed by the `zlib-rs` project that performs as well as `zlib-ng`.

To get the most performance possible, try compiling your program with `RUSTFLAGS='-C target-cpu=native'` to allow `zlib-rs` to use more efficient SIMD instructions.

## Client Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/web-socket-client.rs}}
```

## Server Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/web-socket-server.rs}}
```