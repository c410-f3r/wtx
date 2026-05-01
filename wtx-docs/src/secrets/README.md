# Secrets

The `Secret` struct is a container of sensitive data that needs to be sustained in memory for an extended period. More specifically, it holds locked and encrypted heap-allocated bytes that are decrypted on demand to protect against inspection techniques.

Please keep in mind that this is not a silver bullet, but rather an additional layer of protection. For example, when the `peek` closure is executing, the plaintext secret will exist transiently in CPU registers and caches, which is unavoidable.
 
## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/others/secrets.rs}}
```