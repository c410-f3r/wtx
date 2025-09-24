# HTTP Client Pool

High-level pool of HTTP clients where multiple connections that can be referenced in concurrent scenarios.

Reuses valid connections and recycles dropped communications to minimize contention and latency. Instances are created on-demand and maintained for subsequent requests to the same host.

Also useful because HTTP/2 and HTTP/3 expect long-lived sessions by default unlike HTTP/1.

To use this functionality, it is necessary to activate the `http-client-pool` feature.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/http-client-pool.rs}}
```