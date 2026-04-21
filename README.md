# WTX

[![crates.io][crates-badge]][crates-url]
[![docs][docs-badge]][docs-url]
[![license][license-badge]][license-url]
[![rustc][rustc-badge]][rustc-url]
[![Tests][actions-badge]][actions-url]

[actions-badge]: https://github.com/c410-f3r/wtx/actions/workflows/tests.yaml/badge.svg
[actions-url]: https://github.com/c410-f3r/wtx/actions/workflows/tests.yaml
[crates-badge]: https://img.shields.io/crates/v/wtx.svg?color=blue
[crates-url]: https://crates.io/crates/wtx
[docs-badge]: https://docs.rs/wtx/badge.svg
[docs-url]: https://docs.rs/wtx
[license-badge]: https://img.shields.io/badge/license-MPL2-blue.svg
[license-url]: https://github.com/c410-f3r/wtx/blob/main/LICENSE
[rustc-badge]: https://img.shields.io/badge/rustc-1.95-blue
[rustc-url]: https://blog.rust-lang.org/2025/01/09/Rust-1.95.0.html

A collection of different transport implementations and related tools focused primarily on web technologies. Features the in-house development of 8 IETF RFCs, 3 formal specifications and several other invented ideas.

Works on embedded devices with heap allocators. If you find this crate interesting, please consider giving it a star ⭐ on `GitHub`.

| Specification           | URL                                                              |
| ----------------------- | ---------------------------------------------------------------- |
| `gRPC`                  | <https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md> |
| `HPACK`                 | <https://datatracker.ietf.org/doc/html/rfc7541>                  |
| `HTTP Cookies`          | <https://datatracker.ietf.org/doc/html/rfc6265>                  |
| `HTTP/2`                | <https://datatracker.ietf.org/doc/html/rfc9113>                  |
| `MySQL`                 | <https://dev.mysql.com/doc/dev/mysql-server/latest>              |
| `PostgreSQL`            | <https://www.postgresql.org/docs/current/protocol.html>          |
| `TLS 1.3` (soon)        | <https://datatracker.ietf.org/doc/html/rfc7301>                  |
| `WebSocket`             | <https://datatracker.ietf.org/doc/html/rfc6455>                  |
| `WebSocket Compression` | <https://datatracker.ietf.org/doc/html/rfc7692>                  |
| `WebSocket over HTTP/2` | <https://datatracker.ietf.org/doc/html/rfc8441>                  |
| `X.509`                 | <https://datatracker.ietf.org/doc/html/rfc5280>                  |

## Performance

Many things that generally improve performance are used in the project, to name a few:

1. **Manual vectorization**: When an algorithm is known for processing large amounts of data, several experiments are performed to analyze the best way to split loops in order to allow the compiler to take advantage of SIMD instructions in x86 processors.
2. **Memory allocation**: Whenever possible, all heap allocations are called only once at the start of an instance creation and additionally, stack memory usage is preferably prioritized over heap memory.
3. **Fewer dependencies**: No third-party is injected by default. In other words, additional dependencies are up to the user through the selection of Cargo features, which decreases compilation times. For example, you can see the mere 13 dependencies required by the PostgreSQL client using `cargo tree -e normal --features postgres`.

Since memory are usually held at the instance level instead of being created and dropped on the fly, its usage can growth significantly depending on the use-case. If appropriated, try using a shared pool of resources or try limiting how much data can be exchanged between parties.

## High-level benchmarks

Checkout [wtx-bench](https://c410-f3r.github.io/wtx-bench/) to see a variety of benchmarks or feel free to point any misunderstandings or misconfigurations.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

## Low-level benchmarks

Anything marked with `#[bench]` in the repository is considered a low-level benchmark in the sense that they measure very specific operations that generally serve as the basis for other parts.

Take a look at <https://bencher.dev/perf/wtx> to see all low-level benchmarks over different periods of time.

## Crypto Backend

`WTX` does not have built-in cryptographic algorithms, as such, it is necessary to select a backend when working with `X.509` or `TLS`.

* `crypto-aws-lc-rs`
* `crypto-graviola`
* `crypto-ring`
* `crypto-rust-crypto`

Calling methods will act as NO-OPs if no backend is selected.

## Development benchmarks

These numbers provide an estimate of the expected waiting times when developing with `WTX`. If desired, you can compare them with other similar Rust projects through the `dev-bench.sh` script.

| Technology            | Required Deps [^1] | All Deps [^2]      | Clean Check | Clean Debug Build | Clean Opt Build | Opt size |
| --------------------- | ------------------ | ------------------ | ----------- | ----------------- | --------------- | -------- |
| Client API Framework  | 0                  | 31                 | 6.42s       | 7.79s             | 8.45s           | 872K     |
| gRPC Client           | 2                  | 16                 | 4.80s       | 6.04s             | 6.53s           | 736K     |
| HTTP Client Pool      | 2                  | 15                 | 4.60s       | 5.84s             | 6.44s           | 728K     |
| HTTP Server Framework | 2                  | 34                 | 7.87s       | 10.53s            | 10.60s          | 996K     |
| Postgres Client       | 13                 | 26                 | 5.12s       | 6.19s             | 6.69s           | 652K     |
| WebSocket Client      | 10                 | 22                 | 4.24s       | 5.04s             | 5.31s           | 560K     |

## Examples

Demonstrations of different use-cases can be found in the `wtx-examples` directory as well as in the documentation.

## Limitations

* Does not support systems with a pointer length of 16 bits.

* Expects the infallible sum of the lengths of an arbitrary number of slices, otherwise the program will likely trigger an overflow that can possibly result in unexpected operations. For example, in a 32bit system such a scenario should be viable without swap memory or through specific limiters like `ulimit`.

[^1]: Internal dependencies required by the feature.

[^2]: The sum of optional and required dependencies used by the associated binaries.
