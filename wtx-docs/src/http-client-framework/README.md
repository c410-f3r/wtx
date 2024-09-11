# HTTP Client Framework

High-level pool of HTTP clients that currently only supports HTTP/2. Allows multiple connections that can be referenced in concurrent scenarios.

Activation feature is called `http-client-framework`.

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/http-client-framework.rs}}
```