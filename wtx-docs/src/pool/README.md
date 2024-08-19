# Pool

An asynchronous pool of arbitrary objects where each element is dynamically created or re-created when invalid.

Can also be used for database connections, which is quite handy because it enhances the performance of executing commands and alleviates the use of hardware resources.

Activation feature is called `pool`.

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/examples/pool.rs}}
```