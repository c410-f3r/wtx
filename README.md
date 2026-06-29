# WTX

[![crates.io](https://img.shields.io/crates/v/wtx?color=blue)](https://crates.io/crates/wtx)
[![deps.rs](https://deps.rs/crate/wtx/latest/status.svg)](https://deps.rs/crate/wtx)
[![docs.rs](https://img.shields.io/docsrs/wtx?color=blue)](https://docs.rs/wtx)
[![license](https://img.shields.io/crates/l/wtx?&color=blue)](https://github.com/c410-f3r/wtx/blob/main/LICENSE)
[![rustc](https://img.shields.io/badge/rustc-1.96-blue.svg)](https://blog.rust-lang.org/2025/01/09/Rust-1.96.0.html)
[![tests](https://img.shields.io/github/actions/workflow/status/c410-f3r/wtx/tests.yaml?label=tests)](https://github.com/c410-f3r/wtx/actions/workflows/tests.yaml)

Written by humans. `WTX` is a collection of different transport implementations and related tools focused primarily on web technologies. Features the in-house development of 9 IETF RFCs along side other elements.

Works on embedded devices with heap allocators. If you find this crate interesting, please consider giving it a star ⭐ on `GitHub`.

| Specification            | URL                                                              |
| ------------------------ | ---------------------------------------------------------------- |
| `gRPC`                   | <https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md> |
| `HPACK`                  | <https://datatracker.ietf.org/doc/html/rfc7541>                  |
| `HTTP Cookies`           | <https://datatracker.ietf.org/doc/html/rfc6265>                  |
| `HTTP/2`                 | <https://datatracker.ietf.org/doc/html/rfc9113>                  |
| `PostgreSQL`             | <https://www.postgresql.org/docs/current/protocol.html>          |
| `Raw Public Keys in TLS` | <https://datatracker.ietf.org/doc/html/rfc7250>                        |
| `TLS 1.3`                | <https://datatracker.ietf.org/doc/html/rfc7301>                  |
| `WebSocket`              | <https://datatracker.ietf.org/doc/html/rfc6455>                  |
| `WebSocket Compression`  | <https://datatracker.ietf.org/doc/html/rfc7692>                  |
| `WebSocket over HTTP/2`  | <https://datatracker.ietf.org/doc/html/rfc8441>                  |
| `X.509`                  | <https://datatracker.ietf.org/doc/html/rfc5280>                  |

## High-level benchmarks

Checkout [wtx-bench](https://c410-f3r.github.io/wtx-bench) or [HttpArena](https://www.http-arena.com). Feel free to indicate possible misconfigurations or misunderstandings.

| gRPC                                      | HTTP/2                                        | WebSocket                                  |
| :---------------------------------------: | :-------------------------------------------: | :----------------------------------------: |
| [![HttpArena - gRPC][grpc-img]][grpc-url] | [![HttpArena - HTTP/2][http2-img]][http2-url] | [![HttpArena - WebSocket][ws-img]][ws-url] |

## Low-level benchmarks

Anything marked with `#[bench]` in the repository is considered a low-level benchmark in the sense that they measure very specific operations that generally serve as the basis for other parts.

Take a look at <https://bencher.dev/perf/wtx> to see all low-level benchmarks over different periods of time.

## Development benchmarks

These numbers provide an estimate of the expected waiting times when developing with `WTX`. If desired, you can compare them with other similar Rust projects through the `dev-bench.sh` script.

| Technology              | Required Deps [^1] | All Deps [^2]  | Clean Check | Clean Debug Build | Clean Opt Build | Opt size |
| ----------------------- | ------------------ | -------------- | ----------- | ----------------- | --------------- | -------- |
| gRPC Client             | 2                  | 16             | 4.80s       | 6.04s             | 6.53s           | 736K     |
| HTTP/2 Client Pool      | 2                  | 15             | 4.60s       | 5.84s             | 6.44s           | 728K     |
| HTTP/2 Server Framework | 2                  | 34             | 7.87s       | 10.53s            | 10.60s          | 996K     |
| Postgres Client         | 13                 | 26             | 5.12s       | 6.19s             | 6.69s           | 652K     |
| WebSocket Client        | 10                 | 22             | 4.24s       | 5.04s             | 5.31s           | 560K     |

## Crypto Backend

Taking aside very few exceptions, `WTX` does not have built-in cryptographic algorithms, as such, it is necessary to select a backend when working with features that require them.

* `crypto-aws-lc-rs`
* `crypto-graviola`
* `crypto-openssl`
* `crypto-ring`

Calling methods will halt/panic the application if no backend is selected. These panicking branches will hopefully be erased by dead code analysis if the `crypto` feature is somehow active but never actually used.

In practice many things require cryptography algorithms. For example, `PostgreSQL` uses `HMAC` and secure `HTTP` cookies use `AEAD`.

## Examples

Demonstrations of different use-cases can be found in the `wtx-examples` directory as well as in the documentation located at <https://c410-f3r.github.io/wtx>.

## Limitations

* `WTX` is not widely used and has not undergone security audits, therefore, use it at your own risk. However, reproducible contributions that improve security are always welcome.

* Does not support systems with a pointer length of 16 bits.

* Expects the infallible sum of the lengths of an arbitrary number of slices, otherwise the program will likely trigger an overflow that can possibly result in unexpected operations. For example, in a 32bit system such a scenario should be viable without swap memory or through specific limiters like `ulimit`.

[^1]: Internal dependencies required by the feature.

[^2]: The sum of optional and required dependencies used by the associated binaries.

[grpc-img]: https://cdn.jsdelivr.net/gh/MDA2AV/httparena-badge/httparena-badge-grpc.svg
[grpc-url]: https://www.http-arena.com/#scope=grpc&tuned=0
[http2-img]: https://cdn.jsdelivr.net/gh/MDA2AV/httparena-badge/httparena-badge-h2.svg
[http2-url]: https://www.http-arena.com/#scope=h2&tuned=0
[ws-img]: https://cdn.jsdelivr.net/gh/MDA2AV/httparena-badge/httparena-badge-websocket.svg
[ws-url]: https://www.http-arena.com/#scope=ws&tuned=0