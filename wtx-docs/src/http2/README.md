# HTTP/2

Implementation of [RFC7541](https://datatracker.ietf.org/doc/html/rfc7541) and [RFC9113](https://datatracker.ietf.org/doc/html/rfc9113). HTTP/2 is the second major version of the Hypertext Transfer Protocol, introduced in 2015 to improve web performance, it addresses limitations of HTTP/1.1 while maintaining backwards compatibility.

Passes the `hpack-test-case` and the `h2spec` test suites. Due to official and unofficial deprecations, prioritization and server-push are not supported.

To use this functionality, it is necessary to activate the `http2` feature.

## HTTP/1.1 Upgrade

Does not support upgrading from HTTP/1.1 because browsers also don't support such a feature. Connections must be established directly using HTTP/2 or via ALPN (Application-Layer Protocol Negotiation) during the TLS handshake.

## Operating Modes

There are two distinct operating modes for handling data transmission.

### Automatic Mode

The system takes full responsibility. When you provide a buffer of data to be sent, the implementation automatically fragments it into appropriate `DATA` frames based on the maximum frame size and the current flow control window.

### Manual Mode

Allows more control but you should know HTTP/2 concepts and their interactions like flow control. In this mode the user is responsible for constructing and sending individual `HEADERS`, `DATA` and `TRAILERS` frames.

## Client Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/http2-examples/http2-client.rs}}
```

## Server Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/http2-examples/http2-server.rs}}
```
