
# gRPC

Basic implementation that currently supports only unary calls.

`wtx` does not provide built-in deserialization or serialization utilities capable of manipulate protobuf files. Instead, users are free to choose any third-party that generates Rust bindings and implements the internal `Deserialize` and `Serialize` traits.

Due to the lack of an official parser, the definitions of a `Service` must be manually typed.

Activation feature is called `grpc`.

## Client Example

The bellow snippet requires ~40 dependencies and has an optimized binary size of ~700K.

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/grpc-client-tokio.rs}}
```

## Server Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/grpc-server-tokio-rustls.rs}}
```
