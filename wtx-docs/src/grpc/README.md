# gRPC

Basic implementation that currently only supports unary calls. gRPC is an high-performance remote procedure call framework developed by Google that enables efficient communication between distributed systems, particularly in microservices architectures.

`wtx` does not provide built-in deserialization or serialization utilities capable of manipulate protobuf files. Instead, users are free to choose any third-party that generates Rust bindings and implements the internal `Deserialize` and `Serialize` traits.

Due to the lack of an official parser, the definitions of a `Service` must be manually typed.

Independent benchmarks are available at <https://github.com/LesnyRumcajs/grpc_bench>.

## Client Example

To use this functionality, it is necessary to activate the `grpc-client` feature.

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/grpc-client.rs}}
```

## Server Example

To use this functionality, it is necessary to activate the `grpc-server` feature.

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/grpc-server.rs}}
```
