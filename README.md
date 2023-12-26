# WTX 

[![CI](https://github.com/c410-f3r/wtx/workflows/CI/badge.svg)](https://github.com/c410-f3r/wtx/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/wtx.svg)](https://crates.io/crates/wtx)
[![Documentation](https://docs.rs/wtx/badge.svg)](https://docs.rs/wtx)
[![License](https://img.shields.io/badge/license-APACHE2-blue.svg)](./LICENSE)
[![Rustc](https://img.shields.io/badge/rustc-1.75-lightgray")](https://blog.rust-lang.org/2020/03/12/Rust-1.75.html)

A collection of different transport implementations and related tools focused primarily on web technologies.

Embedded devices that have a heap allocator can use this `no_std` crate.

Documentation is available at <https://c410-f3r.github.io/wtx-site/>.

## Benchmarks

If you disagree with any of the mentioned charts, feel free to checkout [wtx-bench](https://github.com/c410-f3r/wtx/tree/main/wtx-bench) to point any misunderstandings or misconfigurations.

There are mainly 2 things that impact performance, the chosen runtime and the number of pre-allocated bytes. Specially for servers that have to create a new instance for each handshake, pre-allocating a high number of bytes for short-lived or low-transfer connections can have a negative impact.

It is also possible to use libraries that manage pools of resources to avoid having to reconstruct expensive elements all the time.

### PostgreSQL client

![PostgreSQL Benchmark](https://i.imgur.com/vf2tYxY.jpg)

### WebSocket

![WebSocket Benchmark](https://i.imgur.com/Iv2WzJV.jpg)
