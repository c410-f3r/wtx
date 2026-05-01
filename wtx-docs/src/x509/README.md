# X.509

The most utilized format to define public key certificates. X.509 is used in TLS connections, e-mail communication, internal corporate structuring and many other cases.

Implementation of <https://datatracker.ietf.org/doc/html/rfc5280>. Passes a subset of the `x509-limbo` testsuite (<https://github.com/C2SP/x509-limbo>).

To use this functionality, it is necessary to activate the `x509` feature.

## Unsupported

X.509 is huge and was initially issued in 1988, consequently, it contains some elements that are rarely used nowadays. To facilitate development and create convergence, the following items won't be supported as indicated by the CABF (<https://cabforum.org>).

* `Authority Key Identifier`: `authorityCertIssuer` and `authorityCertSerialNumber`.

* `GeneralSubtree`: `minimum` and `maximum`.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/others/x509.rs}}
```