# Contributing

Before submitting a PR, you should probably run `./scripts/internal-tests.sh` and/or `./scripts/integration-tests.sh` to make sure everything is fine.

Integration tests interact with external programs like `podman` or require an internet connection, therefore, they usually aren't good candidates for offline development. On the other hand, internal tests are composed by unit tests, code formatting, `clippy` lints and fuzzing targets.

## Artificial Intelligence

Taking aside very few exceptions like Dependabot, this project only accepts submissions made by humans.

Anyone is free to use AI as a helping tool, however, AI-generated code should only be used as a guide for your own writing. Because of that, contributors also need to understand the implications of their code.

## Building

All rust-related tools will be available once [rustup](https://rustup.rs/) is installed. Additionally, it is necessary to have a C compiler and the development sources of OpenSSL to build the project.

## Submitting PRs

To accelerate and facilitate code review, PRs should contain a minimal description with a minimal amount of commits addressing one feature at the time. Code-related modifications should include corresponding unit, integration or property tests validating the author's intention.

The use of `unsafe` is discourage but when necessary, consider implementing MIRI tests to verify memory safety guarantees. If the introduction of `unsafe` enhances performance, also consider providing `#[bench]` benchmarks.

OPTIONAL: Share a funny or awesome image, music, video or short history.

## Commit's Description

Releases are issued through the use of automating tools, as such, it is important to use `Conventional Commits` (<https://www.conventionalcommits.org>) to allow the creation of more detailed changelogs.