# Contributing

Before submitting a PR, you should probably run `./scripts/internal-tests-all.sh` and/or `./scripts/intergration-tests.sh` to make sure everything is fine.

Integration tests interact with external programs like `podman` or require an internet connection, therefore, they usually aren't good candidates for offline development. On the other hand, internal tests are composed by unit tests, code formatting, `clippy` lints and fuzzing targets.

## Building

Taking aside common Rust tools that can be installed with `rustup` (https://rustup.rs/), at the current time it is only necessary to only have an C compiler to build the project. For example, you can use your favorite system package manager to install `gcc`.

