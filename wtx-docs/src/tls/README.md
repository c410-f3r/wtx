# TLS

TLS 1.3 is the only supported version.

Implementation of [RFC-8446](https://datatracker.ietf.org/doc/html/rfc8446). Passes a subset of the BoringSSL testsuite.

Transport Layer Security (TLS) is a cryptographic protocol that provides secure communication over a computer network by encrypting data to ensure confidentiality, integrity, and authentication. It is widely used in applications such as web browsers ensuring that contents transferred between parties can not be intercepted or altered by unauthorized actors. 

To use this functionality, it is necessary to activate the `tls` feature.

## Plain-text

It is possible to convert a TLS stream into an unencrypted stream through the use of the `TlsModePlainText` structure. In other words, `TlsModePlainText` makes the TLS stream act like a normal plain-text stream ignoring all associated certificates, handshakes and encryptions.

This feature is useful for local tests and also for applications running behind a service mesh that automatically handles mTLS connections. However, `TlsModePlainText` can be \*\*\***DANGEROUS**\*\*\* in a misconfiguration or if you don't know what are you doing, as such, be careful!

## Concurrency

The RFC requires all parties (Client or Server) to send back carefully managed records, such as alerts, if an error occurs.

`WTX` automatically enforces these rules in sequential code but how is the reader part going to access the writer part in concurrent scenarios? In fact, there are numerous ways to approach this and the choice is yours to make.

Examples about possible concurrent utilizations are available in the `wtx-examples` directory.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/examples/tls-client.rs}}
```
