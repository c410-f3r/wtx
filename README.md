# WTX 

[![CI](https://github.com/c410-f3r/wtx/workflows/CI/badge.svg)](https://github.com/c410-f3r/wtx/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/wtx.svg)](https://crates.io/crates/wtx)
[![Documentation](https://docs.rs/wtx/badge.svg)](https://docs.rs/wtx)
[![License](https://img.shields.io/badge/license-APACHE2-blue.svg)](https://github.com/c410-f3r/wtx/blob/main/LICENSE)
[![Rustc](https://img.shields.io/badge/rustc-1.75-lightgray")](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html)

A collection of different transport implementations and related tools focused primarily on web technologies. Contains the implementations of 4 IETF RFCs ([RFC6455](https://datatracker.ietf.org/doc/html/rfc6455), [RFC7541](https://datatracker.ietf.org/doc/html/rfc7541), [RFC7692](https://datatracker.ietf.org/doc/html/rfc7692), [RFC9113](https://datatracker.ietf.org/doc/html/rfc9113)), 1 formal specification ([PostgreSQL](https://www.postgresql.org/docs/16/protocol.html)) and several other invented ideas.

1. [Client API Framework](https://c410-f3r.github.io/wtx/client-api-framework/index.html)
2. [Database Client](https://c410-f3r.github.io/wtx/database/client-connection.html)
3. [Database Object–Relational Mapping](https://c410-f3r.github.io/wtx/database/object%E2%80%93relational-mapping.html)
4. [Database Schema Manager](https://c410-f3r.github.io/wtx/database/schema-management.html)
5. [HTTP2 Client/Server](https://c410-f3r.github.io/wtx/http2/index.html)
6. [Pool Manager](https://c410-f3r.github.io/wtx/pool_manager/index.html)
7. [WebSocket Client/Server](https://c410-f3r.github.io/wtx/web-socket/index.html)

Embedded devices with a working heap allocator can use this `no_std` crate.

## Performance

Many things that generally improve performance are used in the project, to name a few:

1. **Manual vectorization**: When an algorithm is known for processing large amounts of data, several experiments are performed to analyze the best way to split loops in order to allow the compiler to take advantage of SIMD instructions in x86 processors.
2. **Memory allocation**: Whenever possible, all heap allocations are called only once at the start of an instance creation and additionally, stack memory usage is preferably prioritized over heap memory.
3. **Fewer dependencies**: No third-party is injected by default. In other words, additional dependencies are up to the user through the selection of Cargo features, which decreases compilation times. For example, you can see the mere 17 dependencies required by the PostgreSQL client using `cargo tree -e normal --features postgres`.

Since memory are usually held at the instance level instead of being created and dropped on the fly, it is worth noting that its usage can growth significantly depending on the use-case. If appropriated, try using a shared pool of resources or try limiting how much data can be exchanged between parties.

## High-level benchmarks

Checkout [wtx-bench](https://c410-f3r.github.io/wtx-bench/) to see a variety of benchmarks or feel free to point any misunderstandings or misconfigurations.

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpg)

## Low-level benchmarks

Anything marked with `#[bench]` in the repository is considered a low-level benchmark in the sense that they measure very specific operations that generally serve as the basis for other parts.

Take a look at <https://bencher.dev/perf/wtx> to see all low-level benchmarks over different periods of time.

## Limitations

Does not support systems with 16bit memory addresses and expects the infallible addition of the sizes of 8 allocated chunks of memories, otherwise the program will overflow in certain arithmetic operations involving `usize` potentially resulting in unexpected operations.

For example, in a 32bit system you can allocate a maximum of 2^29 bytes of memory for at most 8 elements. Such a scenario should be viable with little swap memory due to the likely triggering of the OOM killer or through specific limiters like `ulimit`.

## Possible future features

* gRPC over HTTP/2 (<https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md>).
* Web server framework
* WebSocket over an HTTP/2 stream (<https://datatracker.ietf.org/doc/html/rfc8441>).
* Static web server
* WebTransport over HTTP/2 (<https://datatracker.ietf.org/doc/draft-ietf-webtrans-http2>).
