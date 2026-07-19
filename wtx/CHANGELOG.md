# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.50.0](https://github.com/c410-f3r/wtx/compare/wtx-v0.49.0...wtx-v0.50.0) - 2026-07-19

### Added

- [**breaking**] Score A+ in testssl

### Other

- Fix BoringSSL tests [4/N]
- Fix BoringSSL tests [3/N]
- Fix BoringSSL tests [2/N]
- Fix BoringSSL tests [1/N]
- Merge pull request #575 from c410-f3r/misc
- Remove OpenSSL

## [0.49.0](https://github.com/c410-f3r/wtx/compare/wtx-v0.48.1...wtx-v0.49.0) - 2026-07-12

### Added

- [**breaking**] Validate subject names in connectors

### Fixed

- Correctly select the key exchange algorithm

## [0.48.1](https://github.com/c410-f3r/wtx/compare/wtx-v0.48.0...wtx-v0.48.1) - 2026-07-10

### Other

- Merge pull request #568 from c410-f3r/misc
- Stop blocking the first thread in the main loop

## [0.48.0](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.9...wtx-v0.48.0) - 2026-07-09

### Added

- [**breaking**] Functional TLS
- [**breaking**] Add native support for PKCS#8
- [**breaking**] Add the `futures` module
- [**breaking**] Let users pass the timestamp that verifies certificates
- [**breaking**] Remove the usage of SuffixPusher in decoders
- feat! Improve frameworks
- [**breaking**] TLS 1.3

### Other

- Fix the features of the examples
- Fix documentation
- Merge pull request #555 from c410-f3r/misc
- Fix more x509 tests
- Update x509-limbo

## [0.47.9](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.8...wtx-v0.47.9) - 2026-06-06

### Added

- return path in websocket optioned server

### Other

- Fix CI

## [0.47.8](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.7...wtx-v0.47.8) - 2026-06-06

### Other

- Merge pull request #549 from c410-f3r/misc
- Include `optioned-server` in `http-server-framework`

## [0.47.7](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.6...wtx-v0.47.7) - 2026-06-05

### Other

- Use socket2

## [0.47.6](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.5...wtx-v0.47.6) - 2026-06-05

### Other

- Merge pull request #545 from c410-f3r/misc
- Send 400 if WS upgrade fails

## [0.47.5](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.4...wtx-v0.47.5) - 2026-06-05

### Other

- Type hint `req`

## [0.47.4](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.3...wtx-v0.47.4) - 2026-06-05

### Added

- Improve Matcher

### Other

- Merge pull request #541 from c410-f3r/misc

## [0.47.3](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.2...wtx-v0.47.3) - 2026-05-30

### Other

- Improve HTTP/2

## [0.47.2](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.1...wtx-v0.47.2) - 2026-05-29

### Added

- add more ReqBuilder methods

## [0.47.1](https://github.com/c410-f3r/wtx/compare/wtx-v0.47.0...wtx-v0.47.1) - 2026-05-28

### Fixed

- byte arrays in pgsql

## [0.47.0](https://github.com/c410-f3r/wtx/compare/wtx-v0.46.1...wtx-v0.47.0) - 2026-05-28

### Added

- [**breaking**] Improve ASCII
- [**breaking**] Add CCADB anchors
- add `RadixTree`

### Other

- Update lints

## [0.46.1](https://github.com/c410-f3r/wtx/compare/wtx-v0.46.0...wtx-v0.46.1) - 2026-05-16

### Fixed

- Read whole ASN.1 bytes in params_oid

### Other

- Merge pull request #529 from c410-f3r/misc

## [0.46.0](https://github.com/c410-f3r/wtx/compare/wtx-v0.45.0...wtx-v0.46.0) - 2026-05-16

### Added

- [**breaking**] Use ASCII in optimization functions

### Fixed

- Bug in SerialNumber

## [0.45.0](https://github.com/c410-f3r/wtx/compare/wtx-v0.44.3...wtx-v0.45.0) - 2026-05-02

### Added

- [**breaking**] Allow the decoding and encoding of arbitrary collections

### Fixed

- Fix PgSQL types

## [0.44.3](https://github.com/c410-f3r/wtx/compare/wtx-v0.44.2...wtx-v0.44.3) - 2026-04-22

### Other

- Allow users to specify HTTP parameters

## [0.44.2](https://github.com/c410-f3r/wtx/compare/wtx-v0.44.1...wtx-v0.44.2) - 2026-04-21

### Other

- Initial automatic publishing
