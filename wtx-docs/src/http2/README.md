# HTTP/2

Implementation of [RFC7541](https://datatracker.ietf.org/doc/html/rfc7541) and [RFC9113](https://datatracker.ietf.org/doc/html/rfc9113). In other words, a low-level HTTP.

Passes the `hpack-test-case` and the `h2spec` test suites. Due to official deprecation, server push and prioritization are not supported.

To use this functionality, it necessary to activate the `http2` feature.

## Client Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/http2-examples/http2-client.rs}}
```

## Server Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/http2-examples/http2-server.rs}}
```