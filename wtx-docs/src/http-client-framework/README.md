# HTTP Client Framework

High-level pool of HTTP clients that currently only supports HTTP/2. Allows multiple connections that can be referenced in concurrent scenarios.

Activation feature is called `http-client-framework`.

## Example

The bellow snippet requires ~25 dependencies and has an optimized binary size of ~700K.

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/http-client-framework-tokio.rs}}
```