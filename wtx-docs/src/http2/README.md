# HTTP/2

Implementation of [RFC7541](https://datatracker.ietf.org/doc/html/rfc7541) and [RFC9113](https://datatracker.ietf.org/doc/html/rfc9113). HTTP/2 is the second major version of the Hypertext Transfer Protocol, introduced in 2015 to improve web performance, it addresses limitations of HTTP/1.1 while maintaining backwards compatibility.

Passes the `hpack-test-case` and the `h2spec` test suites. Due to official deprecation, prioritization is not supported and due to the lack of third-party support, server-push is also not supported.

To use this functionality, it is necessary to activate the `http2` feature.

## Client Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/http2-examples/http2-client.rs}}
```

## Server Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/http2-examples/http2-server.rs}}
```
