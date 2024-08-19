# HTTP Server Framework

A small and fast to compile framework that can interact with many built-in features like PostgreSQL connections.

Activation feature is called `http-server-framework`.

![HTTP/2 Benchmarks](https://i.imgur.com/lUOX3iM.png)

## Example

The bellow snippet requires ~50 dependencies and has an optimized binary size of ~900K.

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/http-server-framework-tokio.rs}}
```