# TLS

Implementation of [RFC-9846](https://datatracker.ietf.org/doc/html/rfc9846). TLS 1.3 is the only supported version.

Transport Layer Security (TLS) is a cryptographic protocol that provides secure communication over a computer network by encrypting data to ensure confidentiality, integrity, and authentication. It is widely used in applications such as web browsers ensuring that contents transferred between parties can not be intercepted or altered by unauthorized actors. 

To use this functionality, it is necessary to activate the `tls` feature.

![WTX - TLS handshake](https://i.imgur.com/Yh8EexK.jpeg)

## Context

TLS contexts dictate how connections should behave, how secret keys should be stored and how signatures should be signed.

It is probably something you shouldn't worry about because most constructors of the `TlsConfig` structure automatically choose the most suitable built-in context. Regardless of that, you can use a different context or create your own.

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/examples/tls-context.rs}}
```

### Plaintext context

Converts a TLS stream into an unencrypted stream, in other words, `PlaintextCtx` makes the TLS stream act like a normal plain-text stream ignoring all associated certificates, handshakes and encryptions.

This feature is useful for local tests and also for applications running behind a service mesh that automatically handles mTLS connections. However, `PlaintextCtx` can be \*\*\***DANGEROUS**\*\*\* in a misconfiguration or if you don't know what are you doing, as such, be careful! 

### Encrypted secret key context

In an ideal world all secret keys should reside in specialized hardware that, when requested, output signatures. The reality however is that such a feature isn't very straightforward to set-up or widely available in cloud providers.

Worse yet, for local deployments a Hardware Security Module (HSM) can cost more than $10000 not to mention the maintenance price.

Is it over for the beta? Perhaps not. There are a bunch of servers running for years using long-lived plaintext secret keys in memory but it is possible to do better.

At the cost of runtime performance, it is possible to keep encrypted secret keys in memory using long pages that are resistant against `RowHammer` or `RAMbleed` and then only decrypt when necessary. That is exactly what the `EncSkCtx` TLS context does using the `Secret` structure.

A silver bullet? No! Better than plaintext data? Definitely!

## Robustness

On its own, the TLS 1.3 RFC is huge, complex and prone to errors. Not to mention other features like ECH or DTLS.

At least for the things supported by `WTX`, all associated `BoringSSL` tests are checked in CI and other tools like `testssl` are also utilized to improve robustness.


## Concurrency

The RFC requires all parties (Client or Server) to send back carefully managed records, such as alerts, if an error occurs.

`WTX` automatically enforces these rules in sequential code but how is the reader part going to access the writer part in concurrent scenarios? In fact, there are numerous ways to approach this and the choice is yours to make.

Examples about possible concurrent utilizations are available in the `wtx-examples` directory.

## Example

```rust,edition2024,no_run
{{#rustdoc_include ../../../wtx-examples/examples/tls-client.rs}}
```
