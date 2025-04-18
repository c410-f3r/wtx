# WTX

[![crates.io][crates-badge]][crates-url]
[![docs][docs-badge]][docs-url]
[![license][license-badge]][license-url]
[![rustc][rustc-badge]][rustc-url]
[![Tests][actions-badge]][actions-url]

[actions-badge]: https://github.com/c410-f3r/wtx/workflows/Tests/badge.svg
[actions-url]: https://github.com/c410-f3r/wtx/actions?query=workflow%3ATests+branch%3Amain
[crates-badge]: https://img.shields.io/crates/v/wtx.svg?color=blue
[crates-url]: https://crates.io/crates/wtx
[docs-badge]: https://docs.rs/wtx/badge.svg
[docs-url]: https://docs.rs/wtx
[license-badge]: https://img.shields.io/badge/license-MPL2-blue.svg
[license-url]: https://github.com/c410-f3r/wtx/blob/main/LICENSE
[rustc-badge]: https://img.shields.io/badge/rustc-1.84-blue
[rustc-url]: https://blog.rust-lang.org/2025/01/09/Rust-1.84.0.html

A collection of different transport implementations and related tools focused primarily on web technologies. Features the in-house development of 6 IETF RFCs ([6265](https://datatracker.ietf.org/doc/html/rfc6265), [6455](https://datatracker.ietf.org/doc/html/rfc6455), [7541](https://datatracker.ietf.org/doc/html/rfc7541), [7692](https://datatracker.ietf.org/doc/html/rfc7692), [8441](https://datatracker.ietf.org/doc/html/rfc8441), [9113](https://datatracker.ietf.org/doc/html/rfc9113)), 3 formal specifications ([gRPC](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md), [MySQL](https://dev.mysql.com/doc/dev/mysql-server/latest/), [PostgreSQL](https://www.postgresql.org/docs/current/protocol.html)) and several other invented ideas.

Embedded devices with a working heap allocator can use this `no_std` crate.

## Comparisons

In a way, `wtx` can be seen as an amalgamation that consolidates the functionality of several other web development projects into a single toolkit. Take a look at the following comparison table to see how some built-from-scratch implementations compare with other similar projects.

| Technology                                         | Similar Projects                                                   | Feature (`wtx`)          |
| -------------------------------------------------- | ------------------------------------------------------------------ | ------------------------ |
| [Client API Framework][client-api-framework]       | N/A                                                                | client-api-framework     |
| [Database Client][database-client]                 | [jdbc][jdbc], [odbc][odbc], [sqlx][sqlx]                           | postgres, mysql          |
| [Database Schema Manager][database-schema-manager] | [flyway][flyway], [liquibase][liquibase], [refinery][refinery]     | schema-manager           |
| [gRPC][grpc]                                       | [grpc][grpc], [tonic][tonic]                                       | grpc-client, grpc-server |
| [HTTP Client Pool][http-client-pool]               | [libcurl][libcurl], [reqwest][reqwest]                             | http-client-pool         |
| [HTTP Server Framework][http-server-framework]     | [axum][axum], [spring-boot][spring-boot], [fastapi][fastapi]       | http-server-framework    |
| [HTTP/2][http2]                                    | [h2][h2], [nghttp2][nghttp2]                                       | http2                    |
| [Pool][pool]                                       | [bb8][bb8], [deadpool][deadpool], [r2d2][r2d2]                     | pool                     |
| [WebSocket][web-socket]                            | [tokio-tungstenite][tokio-tungstenite], [uWebSockets][uWebSockets] | web-socket-handshake     |

Please note that all features are optional and must be set at compile time. For more information, take a look at the documentation available at <https://c410-f3r.github.io/wtx>.

## Performance

Many things that generally improve performance are used in the project, to name a few:

1. **Manual vectorization**: When an algorithm is known for processing large amounts of data, several experiments are performed to analyze the best way to split loops in order to allow the compiler to take advantage of SIMD instructions in x86 processors.
2. **Memory allocation**: Whenever possible, all heap allocations are called only once at the start of an instance creation and additionally, stack memory usage is preferably prioritized over heap memory.
3. **Fewer dependencies**: No third-party is injected by default. In other words, additional dependencies are up to the user through the selection of Cargo features, which decreases compilation times. For example, you can see the mere 16 dependencies required by the PostgreSQL client using `cargo tree -e normal --features postgres`.

Since memory are usually held at the instance level instead of being created and dropped on the fly, it is worth noting that its usage can growth significantly depending on the use-case. If appropriated, try using a shared pool of resources or try limiting how much data can be exchanged between parties.

## High-level benchmarks

Checkout [wtx-bench](https://c410-f3r.github.io/wtx-bench/) to see a variety of benchmarks or feel free to point any misunderstandings or misconfigurations.

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpg)

## Low-level benchmarks

Anything marked with `#[bench]` in the repository is considered a low-level benchmark in the sense that they measure very specific operations that generally serve as the basis for other parts.

Take a look at <https://bencher.dev/perf/wtx> to see all low-level benchmarks over different periods of time.

## Development benchmarks

These numbers provide an estimate of the expected waiting times when developing with `wtx`. If desired, you can compare them with other similar Rust projects through the `dev-bench.sh` script.

| Technology            | Dependencies | Clean Check | Clean Debug Build | Clean Opt Build |
| --------------------- | ------------ | ----------- | ----------------- | --------------- |
| Client API Framework  | 31           | 6.22s       | 7.77s             | 9.45s           |
| gRPC Client           | 16           | 4.81s       | 5.99s             | 7.31s           |
| HTTP Client Pool      | 15           | 4.67s       | 6.04s             | 7.06s           |
| HTTP Server Framework | 37           | 8.17s       | 10.69s            | 11.56s          |
| Postgres Client       | 30           | 5.06s       | 6.10s             | 6.86s           |
| WebSocket Client      | 22           | 4.34s       | 4.92s             | 5.64s           |

All tests were performed on a AMD Ryzen 9 5900X processor.

## Transport Layer Security (TLS)

When using a feature that requires network connection, it is often necessary to perform encrypted communication and since `wtx` is not hard-coded with a specific stream implementation, it is up to you to choose the best TLS provider.

Some utilities like `TokioRustlsConnector` or `TokioRustlsAcceptor` are available to make things more convenient but keep in mind that it is still necessary to activate a crate that provides certificates for client usage.

## Examples

Demonstrations of different use-cases can be found in the `wtx-instances` directory as well as in the documentation.

## Limitations

* Does not support systems with a pointer length of 16 bits.

* Expects the infallible sum of the lengths of an arbitrary number of slices, otherwise the program will likely trigger an overflow that can possibly result in unexpected operations. For example, in a 32bit system such a scenario should be viable without swap memory or through specific limiters like `ulimit`.

[client-api-framework]: https://c410-f3r.github.io/wtx/client-api-framework/index.html
[database-client]: https://c410-f3r.github.io/wtx/database-client/index.html
[database-schema-manager]: https://c410-f3r.github.io/wtx/database-schema-manager/index.html
[grpc]: https://c410-f3r.github.io/wtx/grpc/index.html
[http-client-pool]: https://c410-f3r.github.io/wtx/http-client-pool/index.html
[http-server-framework]: https://c410-f3r.github.io/wtx/http-server-framework/index.html
[http2]: https://c410-f3r.github.io/wtx/http2/index.html
[pool]: https://c410-f3r.github.io/wtx/pool/index.html
[web-socket]: https://c410-f3r.github.io/wtx/web-socket/index.html

[axum]: https://github.com/tokio-rs/axum
[bb8]: https://github.com/djc/bb8
[deadpool]: https://github.com/deadpool-rs/deadpool
[diesel]: https://github.com/diesel-rs/diesel
[fastapi]: https://github.com/fastapi/fastapi
[flyway]: https://github.com/flyway/flyway
[grpc]: https://github.com/grpc/grpc
[h2]: https://github.com/hyperium/h2
[jdbc]: https://docs.oracle.com/javase/8/docs/technotes/guides/jdbc/
[libcurl]: https://github.com/curl/curl
[liquibase]: https://github.com/liquibase/liquibase
[nghttp2]: https://github.com/nghttp2/nghttp2
[odbc]: https://learn.microsoft.com/en-us/sql/odbc
[r2d2]: https://github.com/sfackler/r2d2
[refinery]: https://github.com/rust-db/refinery
[reqwest]: https://github.com/seanmonstar/reqwest
[spring-boot]: https://github.com/spring-projects/spring-boot
[sqlx]: https://github.com/launchbadge/sqlx
[tokio-tungstenite]: https://github.com/snapview/tokio-tungstenite
[tonic]: https://github.com/hyperium/tonic
[uWebSockets]: https://github.com/uNetworking/uWebSockets