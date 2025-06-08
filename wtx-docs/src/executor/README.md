# Executor

Simple dependency-free runtime intended for tests, toy programs and demonstrations. Performance is not a main concern and you should probably use other executors like `tokio`.

To use this functionality, it is necessary to activate the `executor` feature.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/executor.rs}}
```

