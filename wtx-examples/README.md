# WTX - Examples

This crate (`wtx-examples`) provides common auxiliary functionalities for the actual examples located in the `examples` directory but they aren't necessarily meant to be copied verbatim. For example, you should generate or obtain your own valid X.509 certificates for servers.

## TLS

All examples have TLS enabled by default. If you want to work with plaintext for testing purposes, just change `TlsModeVerified` to `TlsModePlainText`. Any associated crypto feature or `TlsConfig` configuration should also be adjusted, for example, it doesn't make sense to add certificates if the connection will always be unencrypted.
