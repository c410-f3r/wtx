# Internal Development

Intended for the development of `WTX` although some tips might be useful for your projects.

## Size constraints

A large enum aggressively used in several places can cause a negative runtime impact. In fact, this is so common that the community created several lints to prevent such a scenario.

- [large_enum_variant](https://rust-lang.github.io/rust-clippy/master/?groups=perf#large_enum_variant)
- [result_large_err](https://rust-lang.github.io/rust-clippy/master/?groups=perf#result_large_err)
- [variant_size_differences](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/types/static.VARIANT_SIZE_DIFFERENCES.html)

Some real-world use-cases and associated benchmarks.

* <https://ziglang.org/download/0.8.0/release-notes.html#Reworked-Memory-Layout>
* <https://github.com/rust-lang/rust/pull/100441>
* <https://github.com/rust-lang/rust/pull/95715>

That is why `WTX` has an enforced `Error` enum size of 16 bytes and that is also the reason why `WTX` has so many bare variants.

## Profiling

Uses the `h2load` benchmarking tool (<https://nghttp2.org/documentation/h2load-howto.htm>l) and the `h2load` internal binary (<https://github.com/c410-f3r/wtx/blob/main/wtx-instances/src/bin/h2load.rs>) for illustration purposes.

### Compilation time / Size

[`cargo-bloat`](https://github.com/RazrFalcon/cargo-bloat): Finds out what takes most of the space in executables.

```bash
cargo bloat --bin h2load --features h2load | head -20
```

[`cargo-llvm-lines`](https://github.com/dtolnay/cargo-llvm-lines): Measures the number and size of instantiations of each generic function in a program.

```bash
CARGO_PROFILE_RELEASE_LTO=fat cargo llvm-lines --bin h2load --features h2load --package wtx-instances --release | head -20
```

### Performance

Prepare the executables in different terminals.

```bash
h2load -c100 --log-file=/tmp/h2load.txt -m10 -n10000 --no-tls-proto=h2c http://localhost:9000
```

```bash
cargo build --bin h2load --features h2load --profile profiling --target x86_64-unknown-linux-gnu
```

[`samply`](https://github.com/mstange/samply): Command line CPU profiler.

```bash
cargo build --bin h2load --features h2load --profile profiling --target x86_64-unknown-linux-gnu
samply record ./target/x86_64-unknown-linux-gnu/profiling/h2load
```

[`callgrind`](https://valgrind.org/docs/manual/cg-manual.html): Gives global, per-function, and per-source-line instruction counts and simulated cache and branch prediction data.

```bash
valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes --simulate-cache=yes ./target/x86_64-unknown-linux-gnu/profiling/h2load
```

## Compiler flags

Some non-standard options that may will influence the final binary.

### Size

* -Cforce-frame-pointers=no
* -Cforce-unwind-tables=no

More size-related parameters can be found at <https://github.com/johnthagen/min-sized-rust>.

### Runtime

* -Cllvm-args=--inline-threshold=1000
* -Cllvm-args=-vectorize-loops
* -Cllvm-args=-vectorize-slp
* -Ctarget-cpu=x86-64-v3

### Security

* -Ccontrol-flow-guard=yes
* -Crelocation-model=pie
* -Crelro-level=full
* -Zstack-protector=strong