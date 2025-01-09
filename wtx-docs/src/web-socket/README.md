# WebSocket

Implementation of [RFC6455](https://datatracker.ietf.org/doc/html/rfc6455) and [RFC7692](https://datatracker.ietf.org/doc/html/rfc7692). WebSocket is a communication protocol that enables full-duplex communication between a client (typically a web browser) and a server over a single TCP connection. Unlike traditional HTTP, which is request-response based, WebSocket allows real-time data exchange without the need for polling.

In-house benchmarks are available at <https://c410-f3r.github.io/wtx-bench>. If you are aware of other benchmark tools, please open a discussion in the GitHub project.

To use this functionality, it is necessary to activate the `web-socket` feature.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

## Autobahn Reports

1. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingclient/index.html" target="_blank">fuzzingclient</a>
2. <a href="https://c410-f3r.github.io/wtx-site/static/fuzzingserver/index.html" target="_blank">fuzzingserver</a>

## Compression

The "permessage-deflate" extension is the only supported compression format and is backed by the `zlib-rs` project that performs as well as `zlib-ng`.

To get the most performance possible, try compiling your program with `RUSTFLAGS='-C target-cpu=native'` to allow `zlib-rs` to use more efficient SIMD instructions.

## No masking

Although not officially endorsed, the `no-masking` parameter described at <https://datatracker.ietf.org/doc/html/draft-damjanovic-websockets-nomasking-02> is supported to increase performance. If such a thing is not desirable, please make sure to check the handshake parameters to avoid accidental scenarios.

To make everything work as intended both parties, client and server, need to implement this feature. For example, web browser won't stop masking frames.

## Client Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/web-socket-examples/web-socket-client.rs}}
```

## Concurrent Client Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/web-socket-examples/web-socket-concurrent-client.rs}}
```

## Server Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/web-socket-examples/web-socket-server.rs}}
```
