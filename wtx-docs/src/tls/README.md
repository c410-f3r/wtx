# TLS

Implementation of TLS 1.3 or [RFC8446](https://datatracker.ietf.org/doc/html/rfc8446). TLS 1.2 **is not** supported.

Transport Layer Security (TLS) is a cryptographic protocol that provides secure communication over a computer network by encrypting data to ensure confidentiality, integrity, and authentication. It is widely used in applications such as web browsers ensuring that contents transferred between parties can not be intercepted or altered by unauthorized actors. 

To use this functionality, it is necessary to activate the `tls` feature.

## Crypto backends

Taking aside very few exceptions, `WTX` does not implement cryptography algorithms like `AES-128-GCM` or `x25519`. That said, it is up to the user to choose the different crypto backends or providers.

* `aws-lc-rs`: <https://github.com/aws/aws-lc-rs>
* `rust-crypto`: <https://github.com/rustcrypto>

## ktls

Kernel TLS improves performance by significantly reducing the need for copying operations between user space and the kernel. However, it is still necessary to select a crypto backend because the handshake occurs in user-land.

Active the `ktls` feature to use this functionality. At the current time only Linux is supported.

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/tls.rs}}
```

## Not supported

* Zero Round Trip Time Resumption (0-RTT)
* PSK-only key establishment (https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.9)
* Key Agreement: ffdhe2048, ffdhe3072, ffdhe4096, ffdhe6144, ffdhe8192, secp521r1, x448
* Signatures: ecdsa_secp521r1_sha512, ecdsa_sha1, ed448, rsa_pkcs1_sha1, rsa_pkcs1_sha512, rsa_pss_pss_sha512