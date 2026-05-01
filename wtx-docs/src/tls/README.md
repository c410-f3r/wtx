# TLS

Implementation of TLS 1.3 or [RFC8446](https://datatracker.ietf.org/doc/html/rfc8446). TLS 1.2 **is not** supported.

Transport Layer Security (TLS) is a cryptographic protocol that provides secure communication over a computer network by encrypting data to ensure confidentiality, integrity, and authentication. It is widely used in applications such as web browsers ensuring that contents transferred between parties can not be intercepted or altered by unauthorized actors. 

To use this functionality, it is necessary to activate the `tls` feature.

## ktls

Kernel TLS improves performance by significantly reducing data copies across the user/kernel boundary and for those interested, offloading to specialized NIC devices is also possible.

`WTX` ***requires*** `ktls` with Linux being the only supported operational system at the current time. Moreover, it is still necessary to select a crypto backend due to user-space handshakes.

## Crypto backends

Taking aside very few exceptions, `WTX` does not implement cryptography algorithms like `AES-256-GCM` or `x25519`. That said, it is up to the user to choose the different crypto backends or providers.

* `aws-lc-rs`: <https://github.com/aws/aws-lc-rs>
* `ring`: <https://github.com/briansmith/ring>
* `rust-crypto`: <https://github.com/rustcrypto>

## Embedded devices

If binary size is a concern, you can activate the `tls-embedded` feature to only enable operations using `x25519` (Key Exchange), `ed25519` (Signing) and `Chacha20Poly1305` (Cipher). `RSA` or other elliptic curves won't be available at compile time.

On a side note, it is worth noting that the search for smaller binaries in embedded systems will likely face challenges as algorithms resistant to quantum computing usually require much larger key sizes.

## Example

```rust,edition2021,no_run
{{#rustdoc_include ../../../wtx-instances/generic-examples/tls.rs}}
```

## Not supported

* Zero Round Trip Time Resumption (0-RTT)
* PSK-only key establishment (https://datatracker.ietf.org/doc/html/rfc8446#section-4.2.9)
* Key Agreement: ffdhe2048, ffdhe3072, ffdhe4096, ffdhe6144, ffdhe8192, secp521r1, x448
* Signatures: ecdsa_secp521r1_sha512, ecdsa_sha1, ed448, rsa_pkcs1_sha1, rsa_pkcs1_sha512, rsa_pss_pss_sha512