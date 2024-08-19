# HTTP/2

Implementation of [RFC7541](https://datatracker.ietf.org/doc/html/rfc7541) and [RFC9113](https://datatracker.ietf.org/doc/html/rfc9113). In other words, a low-level HTTP.

Passes the `hpack-test-case` and the `h2spec` test suites. Due to official deprecation, server push and prioritization are not supported.

Activation feature is called `http2`.

## Client Example

The bellow snippet requires ~25 dependencies and has an optimized binary size of ~700K.

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/http2-client-tokio.rs}}
```

## Server Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/http2-server-tokio-rustls.rs}}
```