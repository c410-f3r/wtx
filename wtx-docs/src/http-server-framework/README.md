# HTTP Server Framework

A small and fast to compile framework that can interact with many built-in features like PostgreSQL connections.

If dynamic or nested routes are needed, then it is necessary to activate the `matchit` feature. Without it, only simple and flat routes will work.

To use this functionality, it necessary to activate the `http-server-framework` feature.

![HTTP/2 Benchmarks](https://i.imgur.com/lUOX3iM.png)

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/http-server-framework-examples/http-server-framework.rs}}
```