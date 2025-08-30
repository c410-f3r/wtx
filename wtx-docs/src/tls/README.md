# TLS

Implementation of [RFC8446](https://datatracker.ietf.org/doc/html/rfc8446). TLS 1.2 is not supported.

Transport Layer Security (TLS) is a cryptographic protocol that provides secure communication over a computer network by encrypting data to ensure confidentiality, integrity, and authentication. It is widely used in applications such as web browsers ensuring that contents transferred between parties can not be intercepted or altered by unauthorized actors. 

To use this functionality, it is necessary to activate the `tls` feature.

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/tls.rs}}
```

## Not supported

* Resumption from pre-TLS 1.3 sessions (https://datatracker.ietf.org/doc/html/rfc8446#section-2.2)