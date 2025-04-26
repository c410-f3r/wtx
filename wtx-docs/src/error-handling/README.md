# Error Handling

The majority of operations performed by `WTX` is fallible, in other words, most functions or methods return a `Result` enum instead of panicking under the hook. A considerable effort is put to hint the compiler that a branch is unreachable to optimize code generation but that is another topic.

Due to this characteristic downstream users are encouraged to create their own `Error` enum with a `WTX` variant along side a `From` trait implementation. Not the mention the unlocking of the useful `?` operator that performs the automatically conversion any supported error element.

```rust2024
extern crate wtx;

use wtx::misc::FromRadix10;

pub enum Error {
    MyDogAteMyHomework,
    RanOutOfCoffee,
    Wtx(wtx::Error)
}

impl From<wtx::Error> for MyCustomErrors {
    fn from(from: wtx::Error) -> Self {
        Self::Wtx(from)
    }
}

fn main() -> Result<(), Error> {
    let _u16_from_bytes = u16::from_radix_10(&[1, 2][..])?;
    let _u16_from_i8 = u16::try_from(1i8).map_err(wtx::Error::from)?;
    Ok(())
}
```

All these conventions are of course optional. If desired everything can be unwrapped using the `Result::unwrap` method.

## Internal size constraint

A large enum aggressively used in several places can cause a negative runtime impact. In fact, this is so common that the community created several lints to prevent such a scenario.

- [large_enum_variant](https://rust-lang.github.io/rust-clippy/master/?groups=perf#large_enum_variant)
- [result_large_err](https://rust-lang.github.io/rust-clippy/master/?groups=perf#result_large_err)
- [variant_size_differences](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_lint/types/static.VARIANT_SIZE_DIFFERENCES.html)

Some real-world use-cases and associated benchmarks.

* https://ziglang.org/download/0.8.0/release-notes.html#Reworked-Memory-Layout
* https://github.com/rust-lang/rust/pull/100441
* https://github.com/rust-lang/rust/pull/95715

That is why `WTX` has an enforced `Error` enum size of 24 bytes that will hopefully get smaller in future and that is also the reason why `WTX` has so many bare variants.

When you encounter an error, try take a look at the available documentation of that specific error (<https://docs.rs/wtx/latest/wtx/enum.Error.html>). If the documentation didn't help, feel free to reach out for potential improvements.

