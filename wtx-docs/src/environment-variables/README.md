# Environment Variables

The `EnvVars` structure allows the reading of environment variables into a custom structure where the name of the fields match the name of the variables. `.env` files are also supported but they should be restricted to development environments.

The unsafe `std::env::set_var` function is not invoked due to concerns about concurrent access, therefore, `std::env::var` should only be used if the associated variable always comes from the current process regardless of the executing environment.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/environment-variables.rs}}
```

