# Secrets

The `Secret` struct is a container for sensitive data that needs to be sustained in memory for an extended period. It ***tries*** to provide an additional layer of protection against speculative execution or cache attacks.

* `Linux`: Uses `memfd_secret`, which practically does not impact performance.
* `Non-Linux`: Holds encrypted heap-allocated bytes that are decrypted on demand. Adds some runtime overhead.

Please keep in mind that this is not a silver bullet, but rather an additional layer of protection. In an ideal world confidential data should be processed in dedicated hardware.

Another thing worth mentioning about non-linux users is that hibernation or swapping to disk can expose plaintext secrets. For example, while the `peek` method is active sensitive data will exist transiently in CPU registers and caches, which is unavoidable.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/examples/secret.rs}}
```