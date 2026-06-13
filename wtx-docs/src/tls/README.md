# TLS

TLS 1.3 is the only supported version.

Implementation of [RFC8446](https://datatracker.ietf.org/doc/html/rfc8446). Passes a subset of the BoringSSL testsuite.

Transport Layer Security (TLS) is a cryptographic protocol that provides secure communication over a computer network by encrypting data to ensure confidentiality, integrity, and authentication. It is widely used in applications such as web browsers ensuring that contents transferred between parties can not be intercepted or altered by unauthorized actors. 

To use this functionality, it is necessary to activate the `tls` feature.

## Not supported

* Zero Round Trip Time Resumption (0-RTT)
* PSK-only key establishment (https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.9)
* Key Agreement: `ffdhe2048`, `ffdhe3072`, `ffdhe4096`, `ffdhe6144`, `ffdhe8192`, `secp521r1`, `x448`.
* Signatures: `ecdsa_secp521r1_sha512`, `ecdsa_sha1`, `ed448`, `rsa_pkcs1_sha1`, `rsa_pkcs1_sha512`, `rsa_pss_pss_sha512`.
* Extensions: `post_handshake_auth` (<https://datatracker.ietf.org/doc/html/rfc8740>).

## Example

The majority of the other examples like WebSocket clients already use TLS connections by default.

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-examples/others/tls.rs}}
```
