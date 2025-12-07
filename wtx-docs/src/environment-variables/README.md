# Environment Variables

`EnvVars` allows the insertion of environment variables into a custom structure where the name of the fields match the name of the variables. `.env` files are also supported but they should be restricted to development environments.

The unsafe `std::env::set_var` function is not invoked due to concerns about concurrent access, therefore, direct usage of `std::env::var` is not recommended unless:

1. `EnvVars` is not used at all.
2. There are no `.env` files.
3. A specific variable is always originated from the current process.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/environment-variables.rs}}
```

