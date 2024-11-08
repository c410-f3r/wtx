# Contributing

Before submitting a PR, you should probably run `./scripts/internal-tests.sh` and/or `./scripts/intergration-tests.sh` to make sure everything is fine.

Integration tests interact with external programs like `podman` or require an internet connection, therefore, they usually aren't good candidates for offline development. On the other hand, internal tests are composed by unit tests, code formatting, `clippy` lints and fuzzing targets.

## Building

Taking aside common Rust tools that can be installed with `rustup` (https://rustup.rs/), at the current time it is only necessary to have an C compiler to build the project. For example, you can use your favorite system package manager to install `gcc`.

## Submitting PRs

To accelerate and facilitate code review, PRs should contain a minimal description with a minimal amount of commits addressing one feature at the time. Code-related modifications should include corresponding unit, integration or property tests validating the intention.

The use of `unsafe` is discourage but when necessary, consider implementing MIRI tests to verify memory safety guarantees. If the introdution of `unsafe` enhances performance, also consider providing `#[bench]` benchmarks.
